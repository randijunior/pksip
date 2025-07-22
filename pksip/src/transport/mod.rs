#![warn(missing_docs)]
//! SIP Transport Layer.
use std::{
    borrow::Cow,
    collections::HashMap,
    io::Write,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime,
};
use bytes::{BufMut, Bytes, BytesMut};
use tokio::sync::mpsc;
use crate::{
    endpoint::Endpoint,
    error::{Error, Result},
    headers::{self, CSeq, CallId, ContentLength, From as FromHdr, Header, Headers, SipHeaderParse, To, Via},
    message::{HostPort, Request, Response, SipMethod, SipMsg, StatusCode, TransportProtocol},
    parser::Parser,
    transaction::{key::TsxKey, server::ServerTransaction, inv_server::InvServerTransaction, ClientTsx, ServerTsx},
};

pub mod tcp;
pub mod udp;
pub mod ws;
mod decoder;

/// This trait represents a abstraction over a SIP transport implementation.
#[async_trait::async_trait]
pub trait Transport: Sync + Send + 'static {
    /// Sends a buffer to the specified remote socket address.
    ///
    /// Returns the number of bytes sent or an I/O error.
    async fn send(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize>;

    /// Returns the transport kind (e.g., UDP, TCP, TLS).
    fn tp_kind(&self) -> TransportProtocol;

    /// Returns the local socket address bound to this transport.
    fn addr(&self) -> SocketAddr;

    /// Checks if the provided address belongs to the same IP address family
    /// (IPv4 vs IPv6) as the local socket address.
    fn is_same_af(&self, addr: &SocketAddr) -> bool {
        let our_addr = self.addr();

        (addr.is_ipv4() && our_addr.is_ipv4()) || (addr.is_ipv6() && our_addr.is_ipv6())
    }

    /// Returns the local transport name.
    fn local_name(&self) -> Cow<'_, str>;

    /// Returns `true` if the transport is reliable (e.g., TCP or TLS).
    fn reliable(&self) -> bool;

    /// Returns `true` if the transport is secure (e.g., TLS).
    fn secure(&self) -> bool;

    /// Returns the key that uniquely identifies this transport connection.
    fn key(&self) -> TransportKey {
        TransportKey::new(self.addr(), self.tp_kind())
    }
}

/// This type represents a key used to identify a transport connection.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TransportKey {
    /// The socket address of the transport.
    addr: SocketAddr,
    /// The transport kind (e.g., UDP, TCP, TLS).
    kind: TransportProtocol,
}

impl TransportKey {
    /// Creates a new `TransportKey`.
    pub fn new(addr: SocketAddr, kind: TransportProtocol) -> Self {
        TransportKey { addr, kind }
    }
}

/// This trait represents a factory for creating SIP transports.
///
/// Normally, this is used by connection oriented transports like TCP and TLS.
#[async_trait::async_trait]
pub trait Factory: Sync + Send {
    /// Creates a new transport instance.
    async fn create(&self, addr: SocketAddr) -> Result<Arc<dyn Transport>>;

    /// Returns the transport protocol this factory creates.
    fn protocol(&self) -> TransportProtocol;
}

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

/// Represents the raw binary content of a message or data block.
///
/// Commonly used for message bodies, network packets, or media content.
#[derive(Clone)]
pub struct Payload(Bytes);

impl Payload {
    /// Creates a new `Payload`.
    #[inline]
    pub fn new(bytes: Bytes) -> Self {
        Payload(bytes)
    }

    /// Returns the raw byte buffer of this payload.
    pub fn buf(&self) -> &[u8] {
        &self.0
    }
}

/// This type represents a SIP packet.
#[derive(Clone)]
pub struct Packet {
    /// The packet payload.
    pub payload: Payload,
    /// The address of the sender.
    pub addr: SocketAddr,
    /// The time the packet was received.
    pub time: SystemTime,
}

/// Represents the address of an outbound message.
pub enum OutgoingAddr {
    /// HostPort address.
    HostPort {
        /// The host and port of the address.
        host: HostPort,
        /// The transport protocol used.
        protocol: TransportProtocol,
    },
    /// SocketAddr address.
    Addr {
        /// The socket address.
        addr: SocketAddr,
        /// The transport to use.
        transport: Arc<dyn Transport>,
    },
}

/// This trait is used to convert a type into a byte buffer.
pub trait ToBytes: Sized {
    /// Converts the type into a byte buffer.
    fn to_bytes(&self) -> Result<Bytes>;
}

/// This type represents an outgoing SIP response.
pub struct OutgoingResponse<'a> {
    /// The SIP response message.
    pub response: Response<'a>,
    /// The address to send the response to.
    pub addr: OutgoingAddr,
    /// The message raw buffer.
    pub buf: Option<Bytes>,
}

impl<'a> OutgoingResponse<'a> {
    /// Returns the message status code.
    pub fn status_code(&self) -> StatusCode {
        self.response.status_line.code
    }

    /// Append headers to the message.
    pub fn append_headers(&mut self, other: &mut Headers<'a>) {
        self.response.append_headers(other);
    }

    /// Returns the message rason text.
    pub fn reason(&self) -> &str {
        self.response.status_line.reason
    }

    /// Returns `true` if this is a provisional response.
    pub fn is_provisional(&self) -> bool {
        self.response.status_line.code.is_provisional()
    }

    /// Set the message body.
    pub fn set_body(&mut self, body: &'a [u8]) {
        self.response.body = Some(body);
    }

    pub fn headers_mut(&mut self) -> &mut Headers<'a> {
        &mut self.response.headers
    }
}

impl ToBytes for OutgoingResponse<'_> {
    fn to_bytes(&self) -> Result<Bytes> {
        let estimated_message_size = if self.response.body.is_none() { 800 } else { 1500 };
        let buf = BytesMut::with_capacity(estimated_message_size);

        let mut buf_writer = buf.writer();

        // Status Line.
        write!(buf_writer, "{}", &self.response.status_line)?;

        // Headers.
        for header in self.response.headers.iter() {
            write!(buf_writer, "{header}\r\n")?;
        }

        // Body.
        if let Some(body) = self.response.body {
            //TODO: write Content-Length
            write!(buf_writer, "\r\n")?;
            buf_writer.write_all(body)?;
        } else {
            write!(buf_writer, "{}: 0\r\n", ContentLength::NAME)?;
            write!(buf_writer, "\r\n")?;
        }
        let out_buffer = buf_writer.into_inner().freeze();

        Ok(out_buffer)
    }
}


/// This type represents an outbound SIP request.
pub struct OutgoingRequest<'a> {
    /// The SIP request message.
    pub msg: Request<'a>,
    /// The addr to send the request to.
    pub addr: SocketAddr,
    /// The message raw buffer.
    pub buf: Option<Bytes>,
    /// The transport to use for sending the request.
    pub transport: Arc<dyn Transport>,
}

impl ToBytes for OutgoingRequest<'_> {
    fn to_bytes(&self) -> Result<Bytes> {
        let estimated_message_size = if self.msg.body.is_none() { 800 } else { 1500 };
        let buf = BytesMut::with_capacity(estimated_message_size);

        let mut buf_writer = buf.writer();

        // Status Line.
        write!(buf_writer, "{}", &self.msg.req_line)?;

        // Headers.
        for h in self.msg.headers.iter() {
            write!(buf_writer, "{h}\r\n")?;
        }

        // Body.
        if let Some(body) = self.msg.body {
            //TODO: write Content-Length
            write!(buf_writer, "\r\n")?;
            buf_writer.write_all(body)?;
        } else {
            write!(buf_writer, "{}: 0\r\n", ContentLength::NAME)?;
            write!(buf_writer, "\r\n")?;
        }
        let out_buffer = buf_writer.into_inner().freeze();

        Ok(out_buffer)
    }
}


pub(crate) struct RequestHeaders<'a> {
    // The topmost Via header as found in the message.
    pub via: Via<'a>,
    // The From header found in the message.
    pub from: headers::From<'a>,
    // The CSeq header as found in the message.
    pub cseq: CSeq,
    // The Call-ID header found in the message.
    pub call_id: CallId<'a>,

    pub to: To<'a>,
}

/// This type represents an received SIP request.
pub struct IncomingRequest<'req> {
    /// The SIP request message.
    pub(crate) request: Request<'req>,
    /// The transport used to receive the request.
    pub(crate) transport: Arc<dyn Transport>,
    /// The packet that contains the request.
    pub(crate) packet: Packet,
    /// The server transaction associated with this request, if any.
    pub(crate) transaction: Option<ServerTsx>,
    /// The request headers extracted from the request.
    pub(crate) request_headers: RequestHeaders<'req>,
}

impl<'req> IncomingRequest<'req> {
    /// Returns the topmost `To` header of the request.
    pub fn to(&self) -> &To {
        &self.request_headers.to
    }
    /// Returns the topmost `Via` header of the request.
    pub fn from(&self) -> &FromHdr {
        &self.request_headers.from
    }

    /// Returns the `Call-ID` header of the request.
    pub fn call_id(&self) -> &CallId {
        &self.request_headers.call_id
    }
    /// Returns the transaction key for this request(if any).
    pub fn tsx_key(&self) -> Option<&TsxKey> {
        self.transaction.as_ref().map(|tsx| tsx.key())
    }

    /// Returns `true` if the message method matches the given `SipMethod`.
    #[inline(always)]
    pub fn is_method(&self, method: &SipMethod) -> bool {
        self.request.method() == method
    }

    /// Returns the message method.
    pub fn method(&self) -> &SipMethod {
        self.request.method()
    }

    /// Gets the source socket address of the packet.
    pub fn addr(&self) -> &SocketAddr {
        &self.packet.addr
    }

    #[inline]
    pub(crate) fn set_tsx_inv(&mut self, tsx: InvServerTransaction) {
        self.transaction = Some(ServerTsx::Invite(tsx));
    }

    #[inline]
    pub(crate) fn set_tsx(&mut self, tsx: ServerTransaction) {
        self.transaction = Some(ServerTsx::NonInvite(tsx));
    }
}

/// Represents an received SIP response.
pub struct IncomingResponse<'r> {
    /// The SIP response message.
    pub(crate) response: Response<'r>,
    /// The transport used to receive the response.
    pub(crate) transport: Arc<dyn Transport>,
    /// The packet that contains the response.
    pub(crate) packet: Packet,
    /// The transaction associated with this response, if any.
    pub(crate) transaction: Option<ClientTsx>,
    /// The request headers extracted from the response.
    pub(crate) request_headers: RequestHeaders<'r>,
}

pub(crate) enum TransportEvent {
    /// A packet was received from the transport layer.
    Packet {
        transport: Arc<dyn Transport>,
        packet: Packet,
    },
    /// A new transport was created.
    Created(Arc<dyn Transport>),
    /// A transport was closed.
    Closed(TransportKey),
    /// A factory was created.
    Factory(Box<dyn Factory>),
}

type TransportTx = mpsc::Sender<TransportEvent>;
type TransportRx = mpsc::Receiver<TransportEvent>;

/// Transport Layer for SIP messages.
pub struct TransportLayer {
    /// A map of transports indexed by their unique keys.
    transports: Mutex<HashMap<TransportKey, Arc<dyn Transport>>>,
    /// A list of transport factories.
    factorys: Mutex<Vec<Box<dyn Factory>>>,
    /// The transport sender used to send events to the transport layer.
    transport_tx: TransportTx,
    /// A receiver for transport events.
    transport_rx: Mutex<Option<TransportRx>>,
}

impl Default for TransportLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TransportLayer {
    pub(crate) fn new() -> Self {
        let (transport_tx, transport_rx) = mpsc::channel(1_000);
        let transport_rx = Mutex::new(Some(transport_rx));

        Self {
            transport_tx,
            transport_rx,
            transports: Default::default(),
            factorys: Default::default(),
            
        }
    }

    pub(crate) fn transport_count(&self) -> usize {
        self.transports.lock().expect("Lock failed").len()
    }

    pub(crate) fn add_transport(&self, transport: Arc<dyn Transport>) {
        self.transports
            .lock()
            .expect("Lock failed")
            .insert(transport.key(), transport);
    }

    pub(crate) fn remove_transport(&self, key: TransportKey) -> Option<Arc<dyn Transport>> {
        self.transports.lock().expect("Lock failed").remove(&key)
    }

    pub(crate) fn add_factory(&self, factory: Box<dyn Factory>) {
        self.factorys.lock().expect("Lock failed").push(factory);
    }

    pub(crate) fn sender(&self) -> &TransportTx {
        &self.transport_tx
    }

    /// Finds a suitable transport for the given destination address and transport type.
    pub fn find(&self, dst: SocketAddr, transport: TransportProtocol) -> Option<Arc<dyn Transport>> {
        log::debug!("Finding suitable transport={} for={}", transport, dst);

        let transports = self.transports.lock().expect("Lock failed");

        // find by remote addr
        let key = TransportKey::new(dst, transport);

        if let Some(transport) = transports.get(&key) {
            return Some(transport.clone());
        }

        // Find by transport protocol and address family
        // TODO: create transport if tcp or tls(find factory)
        transports
            .values()
            .filter(|handle| handle.tp_kind() == transport && handle.is_same_af(&dst))
            .min_by(|a, b| Arc::strong_count(a).cmp(&Arc::strong_count(b)))
            .cloned()
    }

    pub(crate) async fn handle_events(&self, endpoint: &Endpoint) -> Result<()> {
        let mut rx = self.transport_rx.lock().expect("Lock failed").take().unwrap();

        // Loop to receive packets from the transports.
        while let Some(evt) = rx.recv().await {
            match evt {
                TransportEvent::Packet { transport, packet } => {
                    tokio::spawn(Self::on_received_packet(transport, packet, endpoint.clone()));
                }
                TransportEvent::Created(transport) => {
                    self.add_transport(transport);
                }
                TransportEvent::Closed(key) => {
                    self.remove_transport(key);
                }
                TransportEvent::Factory(factory) => {
                    self.add_factory(factory);
                }
            }
        }

        Ok(())
    }

    async fn on_received_packet(transport: Arc<dyn Transport>, packet: Packet, endpoint: Endpoint) -> Result<()> {
        let payload = packet.payload.clone();
        let bytes = payload.buf();

        // Keep-Alive Request packet.
        if bytes == b"\r\n\r\n" {
            transport.send(b"\r\n", &packet.addr).await?;
            return Ok(());
        } else if bytes == b"\r\n" {
            // Keep-Alive Response packet.
            // do nothing
            return Ok(());
        }

        // Parse the packet into an sip message.
        let mut parser = Parser::new(bytes);
        let mut msg = match parser.parse_sip_msg() {
            Ok(parsed_msg) => parsed_msg,
            Err(err) => {
                log::warn!(
                    "Ignoring {} bytes packet from {} {} : {}\n{}-- end of packet.",
                    bytes.len(),
                    transport.tp_kind(),
                    packet.addr,
                    err,
                    String::from_utf8_lossy(bytes)
                );

                return Err(err);
            }
        };

        // Check for mandatory headers.
        let via = msg
            .headers_mut()
            .iter_mut()
            .find_map(|header| if let Header::Via(via) = header { Some(via) } else { None })
            .cloned();

        let Some(mut via) = via else {
            return Err(Error::MissingRequiredHeader(Via::NAME));
        };

        let mut cseq: Option<CSeq> = None;
        let mut from: Option<crate::headers::From> = None;
        let mut call_id: Option<CallId> = None;
        let mut to: Option<To> = None;

        for header in msg.headers().iter() {
            match header {
                Header::From(f) => {
                    from = Some(f.clone());
                }
                Header::To(t) => {
                    to = Some(t.clone());
                }
                Header::CallId(c) => {
                    call_id = Some(c.clone());
                }
                Header::CSeq(c) => {
                    cseq = Some(*c);
                }
                _ => (),
            }
        }

        let Some(from) = from else {
            return Err(Error::MissingRequiredHeader(FromHdr::NAME));
        };

        let Some(to) = to else {
            return Err(Error::MissingRequiredHeader(To::NAME));
        };

        let Some(call_id) = call_id else {
            return Err(Error::MissingRequiredHeader(CallId::NAME));
        };

        let Some(cseq) = cseq else {
            return Err(Error::MissingRequiredHeader(CSeq::NAME));
        };
        // 4. Server Behavior(https://datatracker.ietf.org/doc/html/rfc3581#section-4)
        // The server MUST insert a "received" parameter containing
        // the source IP address that the request came from even if
        // it is identical to the value of the "sent-by" component.
        via.set_received(packet.addr.ip());

        let request_headers = RequestHeaders {
            via,
            cseq,
            call_id,
            from,
            to,
        };
        match msg {
            SipMsg::Request(request) => {
                let mut request = Some(IncomingRequest {
                    request,
                    transport,
                    packet,
                    transaction: None,
                    request_headers,
                });
                endpoint.process_request(&mut request).await?;
            }
            SipMsg::Response(response) => {
                let mut response = Some(IncomingResponse {
                    response,
                    transport,
                    packet,
                    transaction: None,
                    request_headers,
                });
                endpoint.process_response(&mut response).await?;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
/// A trait to start a new transport.
pub(crate) trait TransportStartup {
    async fn start(&self, tx: TransportTx) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use crate::transport::udp::mock::MockUdpTransport;

        use super::*;

        #[test]
        fn test_add_transport() {
            let transports = TransportLayer::default();
            let addr = "127.0.0.1:8080".parse().unwrap();
            let kind = TransportProtocol::Udp;

            transports.add_transport(Arc::new(MockUdpTransport));

            assert!(transports.find(addr, kind).is_some());
            assert!(transports.transport_count() == 1);
        }

        #[test]
        fn test_remove_transport() {
            let transports = TransportLayer::default();
            let udp_tp = Arc::new(MockUdpTransport);
            let addr = "127.0.0.1:8080".parse().unwrap();
            let kind = TransportProtocol::Udp;
            let key = udp_tp.key();

            transports.add_transport(udp_tp);
            assert!(transports.find(addr, kind).is_some());
            assert!(transports.transport_count() == 1);

            transports.remove_transport(key);
            assert!(transports.find(addr, kind).is_none());
            assert!(transports.transport_count() == 0);
        }
    }
}
