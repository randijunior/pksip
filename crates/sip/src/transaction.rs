use async_trait::async_trait;
use tsx_key::TsxKey;

use crate::{
    endpoint::Endpoint,
    message::{SipMethod, StatusCode},
    transport::{
        IncomingMessage, IncomingRequest, MsgBuffer, OutgoingMessage,
        OutgoingResponse, Transport,
    },
};

use std::{
    collections::HashMap,
    io, mem,
    net::SocketAddr,
    ops::Deref,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

pub mod server;
pub mod tsx_key;

/// An estimate of the round-trip time (RTT).
const T1: Duration = Duration::from_millis(500);
/// Maximum retransmission interval for non-INVITE requests and INVITE responses.
const T2: Duration = Duration::from_secs(4);
/// Maximum duration that a message can remain in the network.
const T4: Duration = Duration::from_secs(5);

#[async_trait]
pub trait SipTransaction: Sync + Send + 'static {
    async fn recv_msg(
        &mut self,
        msg: IncomingMessage,
    ) -> io::Result<()>;

    async fn send_msg(
        &mut self,
        msg: OutgoingMessage,
    ) -> io::Result<()>;

    fn terminate(&mut self);
}

/// Represents the inner state of a SIP transaction.
pub struct Inner {
    role: Role,
    endpoint: Endpoint,
    method: SipMethod,
    key: TsxKey,
    transport: Transport,
    addr: SocketAddr,
    state: Mutex<State>,
    status_code: Mutex<Option<StatusCode>>,
    retransmit_count: AtomicUsize,
    last_msg: Mutex<Option<OutgoingResponse>>,
}

#[derive(Clone)]
/// Represents a SIP transaction.
///
/// A SIP transaction consists of a set of messages exchanged between a client (`UAC`) and
/// a server (`UAS`) to complete a certain action, such as establishing or terminating a call.
pub struct Transaction(Arc<Inner>);

impl Transaction {
    fn builder() -> TransactionBuilder {
        Default::default()
    }

    #[inline]
    /// Checks if the transport is reliable.
    pub fn reliable(&self) -> bool {
        self.transport.reliable()
    }

    #[inline]
    /// Retrieves the current state of the transaction.
    pub fn get_state(&self) -> State {
        *self.state.lock().expect("Lock failed")
    }

    #[inline]
    /// Gets the count of retransmissions.
    pub fn retransmission_count(&self) -> u32 {
        self.retransmit_count.load(Ordering::SeqCst) as u32
    }

    #[inline]
    pub fn increment_retransmission_count(&self) -> u32 {
        self.retransmit_count.fetch_add(1, Ordering::SeqCst) as u32
    }

    #[inline]
    /// Retrieves the last status code sent.
    pub fn last_status_code(&self) -> Option<StatusCode> {
        *self.status_code.lock().expect("Lock failed")
    }

    fn on_terminated(&self) {
        self.set_state(State::Terminated);
        self.endpoint.transaction.remove(&self.key);
    }

    fn set_state(&self, state: State) {
        let old = {
            let mut guard = self.state.lock().expect("Lock failed");
            mem::replace(&mut *guard, state)
        };
        log::trace!("State changed from {old:?} to {state:?}");
    }

    #[inline]
    fn set_last_status_code(&self, code: StatusCode) {
        let mut guard = self.status_code.lock().expect("Lock failed");
        *guard = Some(code);
    }

    fn set_last_msg(&self, msg: OutgoingResponse) {
        let mut guard = self.last_msg.lock().expect("Lock failed");
        *guard = Some(msg);
    }

    fn get_last_msg_buf(&self) -> Option<Arc<MsgBuffer>> {
        self.last_msg
            .lock()
            .expect("Lock failed")
            .as_ref()
            .map(|msg| msg.buf.clone().unwrap())
    }

    async fn retransmit(&self) -> io::Result<()> {
        let retransmited = {
            if let Some(msg) = self.get_last_msg_buf() {
                self.transport.send(&msg, &self.addr).await?;
                true
            } else {
                false
            }
        };
        if retransmited {
            self.increment_retransmission_count();
        }
        Ok(())
    }

    async fn send(
        &self,
        mut msg: OutgoingResponse,
    ) -> io::Result<()> {
        log::trace!(
            "Sending Response msg {} {} to {}",
            msg.status_code().into_u32(),
            msg.rphrase(),
            msg.hdrs.cseq.method,
        );
        if msg.buf.is_none() {
            let buf = msg.into_buffer()?;
            msg.buf = Some(buf.into());
        }
        self.transport.send_msg(&self.addr, &msg).await?;
        let code = msg.status_code();
        self.set_last_msg(msg);
        self.set_last_status_code(code);
        Ok(())
    }
}

impl Deref for Transaction {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<(&IncomingRequest, &Endpoint)> for Transaction {
    fn from(
        (request, endpoint): (&IncomingRequest, &Endpoint),
    ) -> Self {
        let key = TsxKey::create(&request);
        let mut builder = Transaction::builder();

        builder.key(key.clone());
        builder.role(Role::UAS);
        builder.endpoint(endpoint.clone());
        builder.method(request.msg.cseq().unwrap().method);
        builder.transport(request.transport().clone());
        builder.addr(request.info.packet().addr);
        builder.state(State::default());

        let tsx = builder.build();
        endpoint.transaction.insert(key, tsx.clone());

        tsx
    }
}

#[derive(Default)]
pub struct TransactionBuilder {
    role: Option<Role>,
    endpoint: Option<Endpoint>,
    method: Option<SipMethod>,
    key: Option<TsxKey>,
    transport: Option<Transport>,
    addr: Option<SocketAddr>,
    state: Option<Mutex<State>>,
    status_code: Option<Mutex<Option<StatusCode>>>,
    last_msg: Option<Mutex<Option<OutgoingResponse>>>,
    retransmit_count: Option<AtomicUsize>,
}

impl TransactionBuilder {
    pub fn role(&mut self, role: Role) -> &mut Self {
        self.role = Some(role);
        self
    }
    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = Some(endpoint);
        self
    }

    pub fn method(&mut self, method: SipMethod) -> &mut Self {
        self.method = Some(method);
        self
    }

    pub fn key(&mut self, key: TsxKey) -> &mut Self {
        self.key = Some(key);
        self
    }

    pub fn transport(&mut self, transport: Transport) -> &mut Self {
        self.transport = Some(transport);
        self
    }

    pub fn addr(&mut self, addr: SocketAddr) -> &mut Self {
        self.addr = Some(addr);
        self
    }

    pub fn state(&mut self, state: State) -> &mut Self {
        self.state = Some(Mutex::new(state));
        self
    }

    pub fn status_code(
        &mut self,
        status_code: Option<StatusCode>,
    ) -> &mut Self {
        self.status_code = Some(Mutex::new(status_code));
        self
    }

    pub fn last_msg(
        &mut self,
        last_msg: Option<OutgoingResponse>,
    ) -> &mut Self {
        self.last_msg = Some(Mutex::new(last_msg));
        self
    }

    pub fn retransmit_count(
        &mut self,
        retransmit_count: usize,
    ) -> &mut Self {
        self.retransmit_count =
            Some(AtomicUsize::new(retransmit_count));
        self
    }

    pub fn build(self) -> Transaction {
        let inner = Inner {
            role: self.role.expect("Role is required"),
            endpoint: self.endpoint.expect("Endpoint is required"),
            method: self.method.expect("Method is required"),
            key: self.key.expect("Key is required"),
            transport: self.transport.expect("Transport is required"),
            addr: self.addr.expect("Address is required"),
            state: self.state.expect("State is required"),
            status_code: self.status_code.unwrap_or_default(),
            last_msg: self.last_msg.unwrap_or_default(),
            retransmit_count: self
                .retransmit_count
                .unwrap_or_default(),
        };
        Transaction(Arc::new(inner))
    }
}

/// The possible roles of a SIP transaction.
pub enum Role {
    /// (User Agent Client): The entity that initiates the request.
    UAC,
    /// (User Agent Server): The entity that responds to the request.
    UAS,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Defines the possible states of a SIP transaction.
pub enum State {
    #[default]
    Trying,
    Proceeding,
    Completed,
    Confirmed,
    Terminated,
}

#[derive(Default)]
pub struct TransactionLayer {
    map: Mutex<HashMap<TsxKey, Transaction>>,
}

impl TransactionLayer {
    #[inline]
    pub fn remove(&self, key: &TsxKey) -> Option<Transaction> {
        let mut map = self.map.lock().expect("Lock failed");
        map.remove(key)
    }

    #[inline]
    fn get(&self, key: &TsxKey) -> Option<Transaction> {
        let map = self.map.lock().expect("Lock failed");
        map.get(key).cloned()
    }

    #[inline]
    pub fn insert(&self, key: TsxKey, tsx: Transaction) {
        let mut map = self.map.lock().expect("Lock failed");
        map.insert(key, tsx);
    }

    /// Find a transaction with the specified key.
    pub fn find_tsx(&self, key: &TsxKey) -> Option<Transaction> {
        self.get(key)
    }

    pub async fn handle_request(
        &self,
        message: IncomingRequest,
    ) -> io::Result<Option<IncomingRequest>> {
        todo!()
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use super::*;

    use std::time::SystemTime;

    use crate::{
        headers::{CSeq, CallId, Headers},
        message::{
            RequestLine, SipRequest, SipResponse, SipUri, StatusCode,
        },
        transport::{
            udp::mock::MockUdpTransport, IncomingInfo, OutgoingInfo,
            Packet, RequestHeaders,
        },
    };

    pub fn response(c: StatusCode) -> OutgoingMessage {
        let from = "sip:alice@127.0.0.1:5060".parse().unwrap();
        let to = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let via = "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200"
            .parse()
            .unwrap();
        let cseq = CSeq {
            cseq: 1,
            method: SipMethod::Options,
        };
        let callid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let hdrs = Box::new(RequestHeaders {
            via,
            from,
            to,
            callid,
            cseq,
        });
        let transport = Transport::new(MockUdpTransport);
        let info = OutgoingInfo {
            addr: transport.addr(),
            transport,
        };
        let msg = SipResponse::new(c.into(), Headers::new(), None);
        let response = OutgoingResponse {
            hdrs,
            msg,
            info,
            buf: None,
        };

        response.into()
    }

    pub fn request(m: SipMethod) -> IncomingMessage {
        let target = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let from = "sip:alice@127.0.0.1:5060".parse().unwrap();
        let to = "sip:bob@127.0.0.1:5060".parse().unwrap();
        let via = "SIP/2.0/UDP 127.0.0.1:5060;branch=z9hG4bK3060200"
            .parse()
            .unwrap();
        let SipUri::Uri(uri) = target else {
            unreachable!()
        };
        let cseq = CSeq { cseq: 1, method: m };
        let callid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
        let hdrs = Box::new(RequestHeaders {
            via,
            from,
            to,
            callid,
            cseq,
        });
        let transport = Transport::new(MockUdpTransport);
        let packet = Packet {
            payload: "".as_bytes().into(),
            addr: transport.addr(),
            time: SystemTime::now(),
        };

        let info = IncomingInfo::new(packet, transport);
        let req_line = RequestLine { method: m, uri };
        let req = SipRequest {
            req_line,
            headers: Headers::default(),
            req_headers: Some(hdrs),
            body: None,
        };
        let incoming = IncomingRequest::new(req, info);

        incoming.into()
    }
}
