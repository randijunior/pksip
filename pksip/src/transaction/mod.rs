#![deny(missing_docs)]
//! SIP Transaction Layer.

use bytes::Bytes;
use client::ClientTransaction;

use inv_server::InvServerTransaction;
use key::TsxKey;
use server::ServerTransaction;

use crate::{
    endpoint::Endpoint,
    error::Result,
    message::{SipMethod, StatusCode},
    transaction::inv_client::InvClientTransaction,
    transport::{IncomingRequest, IncomingResponse, OutgoingRequest, OutgoingResponse, ToBytes, Transport},
    ua::Dialog,
};

use std::{
    collections::HashMap,
    io, mem,
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    time::Duration,
};

pub(crate) mod client;
pub(crate) mod inv_client;
pub(crate) mod inv_server;
pub(crate) mod key;
pub(crate) mod server;

type LastMsg = tokio::sync::RwLock<Option<Bytes>>;
type LastStatusCode = RwLock<Option<StatusCode>>;

#[async_trait::async_trait]
/// A trait for managing the behavior of a SIP transaction.
pub trait Transaction: Sync + Send + 'static {
    /// Estimated round‑trip time (RTT) for message exchanges.
    ///
    /// This value is used as the baseline when computing retransmission intervals.
    const T1: Duration = Duration::from_millis(500);

    /// Maximum retransmission interval for non‑INVITE requests and INVITE responses.
    ///
    /// Retransmissions back off exponentially, but will not exceed this value.
    const T2: Duration = Duration::from_secs(4);

    /// Maximum duration that a message may remain in the network before being discarded.
    ///
    /// Controls the overall lifetime of the transaction, including retransmissions.
    const T4: Duration = Duration::from_secs(5);

    /// Terminate the SIP Transaction.
    fn terminate(&self);
}

struct Inner {
    /// The role of the transaction (UAC or UAS).
    role: Role,
    /// The endpoint associated with the transaction.
    endpoint: Endpoint,
    /// The key used to identify the transaction.
    key: TsxKey,
    /// The transport layer used for communication.
    transport: Arc<dyn Transport>,
    /// The address of the remote endpoint.
    addr: SocketAddr,
    /// The current state of the transaction.
    state: Mutex<State>,
    /// The last status code sent or received in the transaction.
    status_code: LastStatusCode,
    /// The retransmission count for the transaction.
    retransmit_count: AtomicUsize,
    /// The last message sent or received in the
    /// transaction.
    last_msg: LastMsg,

    dialog: RwLock<Option<Dialog>>,
}

#[derive(Clone)]
/// Represents a SIP Transaction.
///
/// A SIP Transaction consists of a set of messages exchanged
/// between a client (`UAC`) and a server (`UAS`) to complete
/// a certain action, such as establishing or terminating a call.
pub struct TransactionInner(Arc<Inner>);

impl TransactionInner {
    pub(crate) fn set_dialog(&self, dialog: Dialog) {
        let mut guard = self.0.dialog.write().expect("Lock failed");
        *guard = Some(dialog);
    }
    fn builder() -> Builder {
        Default::default()
    }

    pub(crate) fn new_tsx_uac(request: &OutgoingRequest, endpoint: &Endpoint, state: State) -> Self {
        let mut builder = Self::builder();

        builder.key(TsxKey::create_client(request));
        builder.role(Role::UAC);
        builder.endpoint(endpoint.clone());
        builder.transport(request.transport.clone());
        builder.addr(request.addr);
        builder.state(state);

        let tsx = builder.build();

        log::trace!("Transaction Created [{:#?}] ({:p})", tsx.0.role, tsx.0);

        tsx
    }

    pub(crate) fn transport(&self) -> &Arc<dyn Transport> {
        &self.0.transport
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        self.0.addr
    }

    pub(crate) fn create_uas(request: &IncomingRequest, endpoint: &Endpoint) -> Self {
        Self::new_tsx_uas(request, endpoint, State::Trying)
    }

    pub(crate) fn create_uas_inv(request: &IncomingRequest, endpoint: &Endpoint) -> Self {
        Self::new_tsx_uas(request, endpoint, State::Initial)
    }

    pub(crate) fn create_uac(request: &OutgoingRequest, endpoint: &Endpoint) -> Self {
        Self::new_tsx_uac(request, endpoint, State::Trying)
    }

    pub(crate) fn create_uac_inv(request: &OutgoingRequest, endpoint: &Endpoint) -> Self {
        Self::new_tsx_uac(request, endpoint, State::Calling)
    }

    pub(crate) fn new_tsx_uas(request: &IncomingRequest, endpoint: &Endpoint, state: State) -> Self {
        let mut builder = Self::builder();

        builder.key(TsxKey::create_server(request));
        builder.role(Role::UAS);
        builder.endpoint(endpoint.clone());
        builder.transport(request.transport.clone());
        builder.addr(request.packet.addr);
        builder.state(state);

        let tsx = builder.build();

        log::trace!("Transaction Created [{:#?}] ({:p})", tsx.0.role, tsx.0);

        tsx
    }

    pub(crate) fn key(&self) -> &TsxKey {
        &self.0.key
    }

    fn schedule_termination(&self, time: Duration) {
        let tsx = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(time).await;
            tsx.on_terminated();
        });
    }

    #[inline]
    /// Checks if the transport is reliable.
    pub fn reliable(&self) -> bool {
        self.0.transport.reliable()
    }

    #[inline]
    /// Retrieves the current state of the Transaction.
    pub fn get_state(&self) -> State {
        *self.0.state.lock().expect("Lock failed")
    }

    #[inline]
    /// Gets the count of retransmissions.
    pub fn retrans_count(&self) -> u32 {
        self.0.retransmit_count.load(Ordering::SeqCst) as u32
    }

    #[inline]
    pub(crate) fn add_retrans_count(&self) -> u32 {
        self.0.retransmit_count.fetch_add(1, Ordering::SeqCst) as u32 + 1
    }

    #[inline]
    /// Retrieves the last status code sent.
    pub fn last_status_code(&self) -> Option<StatusCode> {
        *self.0.status_code.read().expect("Lock failed")
    }

    #[inline]
    /// Retrieves the last msg sent if any.
    pub(crate) async fn last_msg(&self) -> Option<Bytes> {
        self.0.last_msg.read().await.clone()
    }

    fn on_terminated(&self) {
        self.change_state_to(State::Terminated);
        let layer = self.0.endpoint.get_tsx_layer();
        let key = &self.0.key;

        match self.0.role {
            Role::UAC => {
                layer.remove_client_tsx(key);
            }
            Role::UAS => {
                layer.remove_server_tsx(key);
            }
        };
    }

    fn change_state_to(&self, state: State) {
        let old = {
            let mut guard = self.0.state.lock().expect("Lock failed");
            mem::replace(&mut *guard, state)
        };
        log::trace!("State Changed [{old:?} -> {state:?}] ({:p})", self.0);
    }

    #[inline]
    fn set_last_status_code(&self, code: StatusCode) {
        let mut guard = self.0.status_code.write().expect("Lock failed");
        *guard = Some(code);
    }

    pub(crate) async fn set_last_msg(&self, msg: Bytes) {
        let mut guard = self.0.last_msg.write().await;
        *guard = Some(msg);
    }

    pub(crate) fn is_calling(&self) -> bool {
        self.get_state() == State::Calling
    }

    async fn retransmit(&self) -> Result<u32> {
        let retransmited = {
            let lock = self.0.last_msg.read().await;
            if let Some(msg) = lock.as_ref() {
                self.0.transport.send(&msg, &self.0.addr).await?;
                true
            } else {
                false
            }
        };

        if retransmited {
            Ok(self.add_retrans_count())
        } else {
            Err(crate::error::Error::Io(io::Error::new(
                io::ErrorKind::Other,
                "No message to retransmit",
            )))
        }
    }

    async fn tsx_send_request(&self, msg: &mut OutgoingRequest<'_>) -> Result<()> {
        log::debug!("<= Request {} to /{}", msg.msg.req_line.method, msg.addr);
        let buf = msg.buf.take().unwrap_or(msg.to_bytes()?);
        self.0.transport.send(&buf, &self.0.addr).await?;
        self.set_last_msg(buf).await;
        Ok(())
    }

    async fn tsx_send_response(&self, msg: &mut OutgoingResponse<'_>) -> Result<()> {
        let code = msg.status_code();
        log::debug!("=> Response {} {}", code.into_i32(), msg.reason());
        let buf = msg.buf.take().unwrap_or(msg.to_bytes()?);

        self.0.transport.send(&buf, &self.0.addr).await?;
        self.set_last_status_code(code);
        self.set_last_msg(buf).await;
        Ok(())
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        log::trace!(
            "Dropping Transaction [{}] ({:p})",
            self.status_code.read().unwrap().unwrap().into_i32(),
            self
        )
    }
}

#[derive(Default)]
/// Builder for creating a new SIP `Transaction`.
pub struct Builder {
    role: Option<Role>,
    endpoint: Option<Endpoint>,
    key: Option<TsxKey>,
    transport: Option<Arc<dyn Transport>>,
    addr: Option<SocketAddr>,
    state: Option<Mutex<State>>,
    status_code: Option<LastStatusCode>,
    last_msg: Option<LastMsg>,
    retransmit_count: Option<AtomicUsize>,
    dialog: Option<RwLock<Option<Dialog>>>,
}

impl Builder {
    /// Sets the role of the transaction.
    pub fn role(&mut self, role: Role) -> &mut Self {
        self.role = Some(role);
        self
    }

    /// Sets the endpoint associated with the transaction.
    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Sets the key used to identify the transaction.
    pub fn key(&mut self, key: TsxKey) -> &mut Self {
        self.key = Some(key);
        self
    }

    /// Sets the transport associated with the transaction.
    pub fn transport(&mut self, transport: Arc<dyn Transport>) -> &mut Self {
        self.transport = Some(transport);
        self
    }

    /// Sets the address associated with the transaction.
    pub fn addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    /// Sets the transaction state.
    pub fn state(&mut self, state: State) -> &mut Self {
        self.state = Some(Mutex::new(state));
        self
    }

    /// Set the status code.
    pub fn status_code(&mut self, status_code: Option<StatusCode>) -> &mut Self {
        self.status_code = Some(RwLock::new(status_code));
        self
    }

    /// Set the retransmission count.
    pub fn retransmit_count(&mut self, retransmit_count: usize) -> &mut Self {
        self.retransmit_count = Some(AtomicUsize::new(retransmit_count));
        self
    }

    /// Finalize the builder into a `Transaction`.
    pub fn build(self) -> TransactionInner {
        let inner = Inner {
            role: self.role.expect("Role is required"),
            endpoint: self.endpoint.expect("Endpoint is required"),
            key: self.key.expect("Key is required"),
            transport: self.transport.expect("Arc<dyn Transport> is required"),
            addr: self.addr.expect("Address is required"),
            state: self.state.expect("State is required"),
            status_code: self.status_code.unwrap_or_default(),
            last_msg: self.last_msg.unwrap_or_default(),
            retransmit_count: self.retransmit_count.unwrap_or_default(),
            dialog: self.dialog.unwrap_or_default(),
        };

        TransactionInner(Arc::new(inner))
    }
}

/// The possible roles of a SIP Transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// (User Agent Client): The entity that initiates the
    /// request.
    UAC,
    /// (User Agent Server): The entity that responds to the
    /// request.
    UAS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Defines the possible states of a SIP Transaction.
pub enum State {
    #[default]
    /// Initial state
    Initial,
    /// Calling state
    Calling,
    /// Trying state
    Trying,
    /// Proceeding state
    Proceeding,
    /// Completed state
    Completed,
    /// Confirmed state
    Confirmed,
    /// Terminated state
    Terminated,
}

#[derive(Clone)]
/// An Server Transaction, either Invite or NonInvite.
pub enum ServerTsx {
    /// An NonInvite Server Transaction.
    NonInvite(ServerTransaction),
    /// An Invite Server Transaction.
    Invite(InvServerTransaction),
}

impl ServerTsx {
    /// Attach a dialog to the transaction.
    pub fn set_dialog(&self, dialog: Dialog) {
        match self {
            ServerTsx::NonInvite(uas) => uas.set_dialog(dialog),
            ServerTsx::Invite(uas_inv) => uas_inv.set_dialog(dialog),
        }
    }
    /// Retrieves the last status code sent by transaction.
    pub fn last_status_code(&self) -> Option<StatusCode> {
        match self {
            ServerTsx::NonInvite(uas) => uas.last_status_code(),
            ServerTsx::Invite(uas_inv) => uas_inv.last_status_code(),
        }
    }

    pub(crate) fn key(&self) -> &TsxKey {
        match self {
            ServerTsx::NonInvite(uas) => uas.key(),
            ServerTsx::Invite(uas_inv) => uas_inv.key(),
        }
    }

    pub(crate) async fn receive_request(&self, request: &IncomingRequest<'_>) -> Result<()> {
        match self {
            ServerTsx::NonInvite(uas) => {
                if matches!(uas.get_state(), State::Proceeding | State::Completed) {
                    uas.retransmit().await?;
                }
                Ok(())
            }
            ServerTsx::Invite(uas_inv) => {
                match uas_inv.get_state() {
                    State::Completed if request.is_method(&SipMethod::Ack) => {
                        uas_inv.change_state_to(State::Confirmed);
                        let mut lock = uas_inv.tx_confirmed.lock().expect("Lock failed");
                        if let Some(sender) = lock.take() {
                            sender.send(()).unwrap();
                        }
                        drop(lock);
                        uas_inv.terminate();
                    }
                    State::Proceeding => {
                        uas_inv.retransmit().await?;
                    }
                    _ => (),
                }
                Ok(())
            }
        }
    }
}

impl From<ServerTransaction> for ServerTsx {
    fn from(tsx: ServerTransaction) -> Self {
        ServerTsx::NonInvite(tsx)
    }
}

impl From<InvServerTransaction> for ServerTsx {
    fn from(tsx: InvServerTransaction) -> Self {
        ServerTsx::Invite(tsx)
    }
}

#[derive(Clone)]
/// An Client Transaction, either Invite or NonInvite.
pub enum ClientTsx {
    /// An NonInvite Client Transaction.
    NonInvite(client::ClientTransaction),
    /// An Invite Client Transaction.
    Invite(inv_client::InvClientTransaction),
}

#[derive(Default)]
/// Represents the transaction layer of the SIP protocol.
///
/// This type holds all server and client transactions created
/// by the TU (Transaction User).
pub struct TransactionLayer {
    client_transactions: Mutex<HashMap<TsxKey, ClientTsx>>,
    server_transactions: Mutex<HashMap<TsxKey, ServerTsx>>,
}

impl TransactionLayer {
    /// Remove an server transaction in the collection.
    #[inline]
    pub fn remove_server_tsx(&self, key: &TsxKey) -> Option<ServerTsx> {
        let mut map = self.server_transactions.lock().expect("Lock failed");
        map.remove(key)
    }

    /// Remove an client transaction in the collection.
    #[inline]
    pub fn remove_client_tsx(&self, key: &TsxKey) -> Option<ClientTsx> {
        let mut map = self.client_transactions.lock().expect("Lock failed");
        map.remove(key)
    }

    #[inline]
    pub(crate) fn add_server_tsx_to_map(&self, tsx: ServerTransaction) {
        let key = tsx.0.key.clone();
        let mut map = self.server_transactions.lock().expect("Lock failed");

        map.insert(key, ServerTsx::NonInvite(tsx));
    }

    #[inline]
    pub(crate) fn add_client_tsx_to_map(&self, tsx: ClientTransaction) {
        let key = tsx.key().clone();
        let mut map = self.client_transactions.lock().expect("Lock failed");

        map.insert(key, ClientTsx::NonInvite(tsx));
    }

    #[inline]
    pub(crate) fn add_client_inv_tsx_to_map(&self, tsx: InvClientTransaction) {
        let key = tsx.key().clone();
        let mut map = self.client_transactions.lock().expect("Lock failed");

        map.insert(key, ClientTsx::Invite(tsx));
    }

    #[inline]
    pub(crate) fn add_server_tsx_inv_to_map(&self, tsx: InvServerTransaction) {
        let key = tsx.0.key.clone();
        let mut map = self.server_transactions.lock().expect("Lock failed");

        map.insert(key, ServerTsx::Invite(tsx));
    }

    fn find_server_tsx(&self, key: &TsxKey) -> Option<ServerTsx> {
        self.server_transactions.lock().expect("Lock failed").get(key).cloned()
    }

    fn find_client_tsx(&self, key: &TsxKey) -> Option<ClientTsx> {
        self.client_transactions.lock().expect("Lock failed").get(key).cloned()
    }

    pub(crate) async fn handle_response(&self, response: &IncomingResponse<'_>) -> Result<bool> {
        let cseq_method = response.request_headers.cseq.method();
        let via_branch = response.request_headers.via.branch().unwrap();

        let key = TsxKey::create_client_with(cseq_method, via_branch);
        let client_tsx = {
            match self.find_client_tsx(&key) {
                Some(tsx) => tsx,
                None => return Ok(false),
            }
        };
        let handled = match client_tsx {
            ClientTsx::NonInvite(tsx) => tsx.receive(response).await?,
            ClientTsx::Invite(tsx_inv) => tsx_inv.receive(response).await?,
        };

        Ok(handled)
    }

    pub(crate) async fn handle_request(&self, request: &IncomingRequest<'_>) -> Result<bool> {
        let server_tsx = {
            let key = TsxKey::create_server(request);

            match self.find_server_tsx(&key) {
                Some(tsx) => tsx,
                None => return Ok(false),
            }
        };

        server_tsx.receive_request(request).await?;
        Ok(true)
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use super::*;
    use crate::headers::Header;

    use std::time::SystemTime;

    use crate::{
        headers::{CSeq, CallId, Headers, SipHeaderParse},
        message::{Request, RequestLine, Response, SipMethod, SipUri},
        transport::{udp::mock::MockUdpTransport, OutgoingAddr, Packet, Payload, RequestHeaders},
    };

    pub fn response<'a>(c: StatusCode) -> OutgoingResponse<'a> {
        let from = crate::headers::From::from_bytes("sip:alice@127.0.0.1:5060".as_bytes()).unwrap();
        let to = crate::headers::To::from_bytes("sip:bob@127.0.0.1:5060".as_bytes()).unwrap();
        let via = crate::headers::Via::from_bytes(
            "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200;received=127.0.0.1".as_bytes(),
        )
        .unwrap();

        let cseq = crate::headers::Header::CSeq(CSeq::new(1, SipMethod::Options));
        let callid = crate::headers::Header::CallId(CallId::new("bs9ki9iqbee8k5kal8mpqb"));
        let mut headers = Headers::new();

        headers.push(crate::headers::Header::Via(via));
        headers.push(crate::headers::Header::From(from));
        headers.push(crate::headers::Header::To(to));
        headers.push(callid);
        headers.push(cseq);

        let transport = Arc::new(MockUdpTransport);
        let addr = OutgoingAddr::Addr {
            addr: transport.addr(),
            transport,
        };
        let mut response = Response::new(crate::message::StatusLine {
            code: c,
            reason: c.reason().into(),
        });

        response.headers = headers;

        OutgoingResponse {
            response,
            addr,
            buf: None,
        }
    }

    pub fn request<'a>(m: SipMethod) -> IncomingRequest<'a> {
        let from = crate::headers::From::from_bytes("sip:alice@127.0.0.1:5060".as_bytes()).unwrap();
        let to = crate::headers::To::from_bytes("sip:bob@127.0.0.1:5060".as_bytes()).unwrap();
        let p = &mut crate::parser::Parser::new("sip:bob@127.0.0.1:5060".as_bytes());
        let target = p.parse_sip_uri(false).unwrap();
        let via = crate::headers::Via::from_bytes(
            "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200;received=127.0.0.1".as_bytes(),
        )
        .unwrap();
        let SipUri::Uri(uri) = target else { unreachable!() };
        let cseq = CSeq::new(1, m);
        let call_id = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let transport = Arc::new(MockUdpTransport);
        let packet = Packet {
            payload: Payload::new(bytes::Bytes::new()),
            addr: transport.addr(),
            time: SystemTime::now(),
        };

        let req_line = RequestLine { method: m, uri };
        let req = Request {
            req_line,
            headers: Headers::default(),
            body: None,
        };
        let incoming = IncomingRequest {
            request: req,
            transport,
            packet,
            transaction: None,
            request_headers: RequestHeaders {
                to,
                cseq,
                via,
                call_id,
                from,
            },
        };

        incoming
    }

    pub fn outgoing_request<'o>(m: SipMethod) -> OutgoingRequest<'o> {
        let from = crate::headers::From::from_bytes("sip:alice@127.0.0.1:5060".as_bytes()).unwrap();
        let p = &mut crate::parser::Parser::new("sip:bob@127.0.0.1:5060".as_bytes());
        let target = p.parse_sip_uri(false).unwrap();
        let via = crate::headers::Via::from_bytes(
            "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200;received=127.0.0.1".as_bytes(),
        )
        .unwrap();
        let SipUri::Uri(uri) = target else { unreachable!() };
        let cseq = CSeq::new(1, m);
        let call_id = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let transport = Arc::new(MockUdpTransport);

        let mut headers = Headers::with_capacity(4);

        headers.push(Header::From(from));
        headers.push(Header::Via(via));
        headers.push(Header::CSeq(cseq));
        headers.push(Header::CallId(call_id));

        let req_line = RequestLine { method: m, uri };
        let req = Request {
            req_line,
            headers,
            body: None,
        };

        let outgoing = OutgoingRequest {
            msg: req,
            addr: transport.addr(),
            buf: None,
            transport,
        };

        outgoing
    }

    pub fn incoming_response<'r>(c: StatusCode) -> IncomingResponse<'r> {
        let from = crate::headers::From::from_bytes("sip:alice@127.0.0.1:5060".as_bytes()).unwrap();
        let to = crate::headers::To::from_bytes("sip:bob@127.0.0.1:5060".as_bytes()).unwrap();
        let via = crate::headers::Via::from_bytes(
            "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200;received=127.0.0.1".as_bytes(),
        )
        .unwrap();

        let cseq = CSeq::new(1, SipMethod::Options);
        let call_id = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let mut headers = Headers::new();

        headers.push(crate::headers::Header::Via(via.clone()));
        headers.push(crate::headers::Header::From(from.clone()));
        headers.push(crate::headers::Header::To(to.clone()));
        headers.push(crate::headers::Header::CallId(call_id.clone()));
        headers.push(crate::headers::Header::CSeq(cseq));

        let transport = Arc::new(MockUdpTransport);
        let addr = transport.addr();
        let mut response = Response::new(crate::message::StatusLine {
            code: c,
            reason: c.reason().into(),
        });
        response.headers = headers;

        IncomingResponse {
            response,
            transport,
            packet: Packet {
                payload: Payload::new(bytes::Bytes::new()),
                addr: addr,
                time: SystemTime::now(),
            },
            transaction: None,
            request_headers: RequestHeaders {
                to,
                via,
                cseq,
                call_id,
                from,
            },
        }
    }

    pub async fn default_endpoint() -> Endpoint {
        let endpoint = crate::endpoint::Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;

        endpoint
    }
}

#[cfg(test)]
mod tests {
    use crate::{endpoint, message::SipMethod};

    use super::*;

    #[tokio::test]
    async fn test_non_invite_server_tsx() {
        let mut req = mock::request(SipMethod::Register);

        let endpoint = endpoint::Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;

        endpoint.new_uas_tsx(&mut req);

        let transactions = endpoint.get_tsx_layer();
        let key = req.tsx_key().unwrap().clone();
        let tsx = transactions.find_server_tsx(&key);

        assert!(matches!(tsx.as_ref(), Some(ServerTsx::NonInvite(_))));
        let tsx = match tsx.unwrap() {
            ServerTsx::NonInvite(tsx) => tsx,
            _ => unreachable!(),
        };

        tsx.on_terminated();
        let tsx = transactions.find_server_tsx(&key);

        assert!(tsx.is_none());
    }

    #[tokio::test]
    async fn test_invite_server_tsx() {
        let mut req = mock::request(SipMethod::Invite);

        let endpoint = endpoint::Builder::new()
            .with_transaction_layer(TransactionLayer::default())
            .build()
            .await;

        endpoint.new_uas_inv_tsx(&mut req);

        let transactions = endpoint.get_tsx_layer();
        let key = req.tsx_key().unwrap().clone();

        let tsx = transactions.find_server_tsx(&key);

        assert!(matches!(tsx.as_ref(), Some(ServerTsx::Invite(_))));

        let tsx = match tsx.unwrap() {
            ServerTsx::Invite(tsx) => tsx,
            _ => unreachable!(),
        };

        tsx.on_terminated();

        let tsx = transactions.find_server_tsx(&key);

        assert!(tsx.is_none());
    }
}
