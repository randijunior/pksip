#![warn(missing_docs)]
//! SIP Transport Layer.
use std::{
    cmp::Ordering,
    collections::HashMap,
    io::Write,
    net::SocketAddr,
    ops::Deref,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use tokio::sync::mpsc;
use udp::UdpTransport;

use crate::{
    endpoint::Endpoint,
    error::{Error, Result},
    headers::{CSeq, CallId, ContentLength, From as FromHdr, Header, Headers, SipHeaderParse, To, Via},
    message::{buffer::Buffer, HostPort, Method, Request, Response, SipMsg, StatusCode, TransportKind},
    parser::ParseCtx,
    transaction::{client::TsxUac, client_inv::TsxUacInv, key::TsxKey, ClientTsx, ServerTsx},
};

pub mod tcp;
pub mod udp;
pub mod ws;

mod decoder;

pub(crate) enum TransportEvent {
    /// A packet was received from the transport layer.
    PacketReceived(TransportPacket),

    /// A new transport was created.
    TransportCreated(Transport),

    /// A transport was closed.
    TransportClosed(TransportKey),

    FactoryCreated(Box<dyn Factory>),
}

type TransportPacket = (Transport, Packet);

type TransportTx = mpsc::Sender<TransportEvent>;
type TransportRx = mpsc::Receiver<TransportEvent>;

#[async_trait::async_trait]
/// This trait represents a abstraction over a SIP transport layer implementation.
pub trait SipTransport: Sync + Send + 'static {
    /// Sends a buffer to the specified remote socket address.
    ///
    /// Returns the number of bytes sent or an I/O error.
    async fn send(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize>;

    /// Returns the transport kind (e.g., UDP, TCP, TLS).
    fn tp_kind(&self) -> TransportKind;

    /// Returns the local socket address bound to this transport.
    fn addr(&self) -> SocketAddr;

    /// Checks if the provided address belongs to the same IP address family
    /// (IPv4 vs IPv6) as the local socket address.
    fn is_same_address_family(&self, addr: &SocketAddr) -> bool {
        let our_addr = self.addr();

        (addr.is_ipv4() && our_addr.is_ipv4()) || (addr.is_ipv6() && our_addr.is_ipv6())
    }

    /// Returns the local transport name.
    fn local_name(&self) -> std::borrow::Cow<'_, str>;

    /// Returns `true` if the transport is reliable (e.g., TCP or TLS).
    fn reliable(&self) -> bool;

    /// Returns `true` if the transport is secure (e.g., TLS).
    fn secure(&self) -> bool;

    /// Returns the key that uniquely identifies this transport connection.
    fn key(&self) -> TransportKey {
        TransportKey::new(self.addr(), self.tp_kind())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// Key used to identify a transport connection.
pub struct TransportKey {
    addr: SocketAddr,
    kind: TransportKind,
}

impl TransportKey {
    /// Creates a new `TransportKey`.
    pub fn new(addr: SocketAddr, kind: TransportKind) -> Self {
        TransportKey { addr, kind }
    }
}

#[async_trait::async_trait]
/// This trait represents a factory for creating SIP transports.
///
/// Normally, this is used by connection oriented transports like TCP and TLS.
pub trait Factory: Sync + Send {
    /// Creates a new transport instance.
    async fn create(&self, addr: SocketAddr) -> Result<Transport>;

    /// Returns the transport kind this factory creates.
    fn transport_kind(&self) -> TransportKind;
}

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

#[derive(Clone)]
/// Represents the raw binary content of a message or data block.
///
/// Commonly used for message bodies, network packets, or media content.
pub(crate) struct Payload(bytes::Bytes);

impl Payload {
    /// Creates a new `Payload`.
    #[inline]
    pub fn new(bytes: bytes::Bytes) -> Self
    {
        Payload(bytes)
    }

    pub fn buf(&self) -> &[u8] {
        &self.0
    }
}


#[derive(Clone)]
/// This type represents a SIP packet.
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
        protocol: TransportKind,
    },
    /// SocketAddr address.
    Addr {
        /// The socket address.
        addr: SocketAddr,
        /// The transport to use.
        transport: Transport,
    },
}

/// This type represents an outgoing SIP response.
pub struct OutgoingResponse<'a> {
    /// The SIP response message.
    pub msg: Response<'a>,

    /// The address to send the response to.
    pub addr: OutgoingAddr,

    /// The message raw buffer.
    pub buf: Option<Buffer>,
}

impl<'a> OutgoingResponse<'a> {
    /// Encode this message to a buffer.
    pub fn encode(&self) -> Result<Buffer> {
        let estimated_message_size = if self.msg.body.is_none() { 800 } else { 1500 };
        let mut buf = Buffer::with_capacity(estimated_message_size);

        // Status Line.
        write!(buf, "{}", &self.msg.status_line)?;

        // Headers.
        for h in self.msg.headers.iter() {
            write!(buf, "{h}\r\n")?;
        }

        // Body.
        if let Some(body) = &self.msg.body {
            //TODO: write Content-Length
            write!(buf, "\r\n")?;
            buf.extend_from_slice(body);
        } else {
            write!(buf, "{}: 0\r\n", ContentLength::NAME)?;
            write!(buf, "\r\n")?;
        }

        Ok(buf)
    }

    /// Returns the message status code.
    pub fn status_code(&self) -> StatusCode {
        self.msg.status_line.code
    }

    /// Returns the message rason text.
    pub fn reason(&self) -> &str {
        self.msg.status_line.reason
    }

    /// Returns `true` if this is a provisional response.
    pub fn is_provisional(&self) -> bool {
        self.msg.status_line.code.is_provisional()
    }

    /// Append headers to the message.
    pub fn append_headers(&mut self, other: &mut Headers<'a>) {
        self.msg.append_headers(other);
    }

    /// Set the message body.
    pub fn set_body(&mut self, body: &'a [u8]) {
        self.msg.body = Some(body);
    }
}

/// This type represents an outbound SIP request.
pub struct OutgoingRequest<'a> {
    /// The SIP request message.
    pub msg: Request<'a>,

    /// The addr to send the request to.
    pub addr: SocketAddr,

    /// The message raw buffer.
    pub buf: Option<Arc<Buffer>>,

    pub(crate) transport: Transport,

    pub(crate) tsx: Option<ClientTsx>,

    pub(crate) req_headers: RequestHeaders<'a>,
}

impl OutgoingRequest<'_> {
    pub(crate) fn set_tsx(&mut self, tsx: TsxUac) {
        self.tsx = Some(ClientTsx::NonInvite(tsx));
    }

    pub(crate) fn set_inv_tsx(&mut self, tsx: TsxUacInv) {
        self.tsx = Some(ClientTsx::Invite(tsx));
    }
}



pub(crate) struct RequestHeaders<'a> {
    // The topmost Via header as found in the message.
    pub via: Via<'a>,

    // The CSeq header as found in the message.
    pub cseq: CSeq,

    // The Call-ID header found in the message.
    pub call_id: CallId<'a>,
}

/// This type represents an received SIP request.
pub struct IncomingRequest<'a> {
    pub(crate) msg: Request<'a>,

    pub(crate) transport: Transport,
    pub(crate) packet: Packet,

    pub(crate) tsx: Option<ServerTsx>,

    pub(crate) req_headers: RequestHeaders<'a>,
}

impl<'a> IncomingRequest<'a> {
    /// Returns the transaction key for this request(if any).
    pub fn tsx_key(&self) -> Option<&TsxKey> {
        self.tsx.as_ref().map(|tsx| tsx.key())
    }

    /// Returns `true` if the message method matches the given `Method`.
    pub fn is_method(&self, method: &Method) -> bool {
        self.msg.method() == method
    }

    /// Returns the message method.
    pub fn method(&self) -> &Method {
        self.msg.method()
    }

    /// Gets the source socket address of the packet.
    pub fn addr(&self) -> &SocketAddr {
        &self.packet.addr
    }
}

/// Represents an received SIP response.
pub struct IncomingResponse<'a> {
    pub(crate) msg: Response<'a>,

    pub(crate) transport: Transport,

    pub(crate) packet: Packet,
}

impl IncomingResponse<'_> {
    pub fn is_final(&self) -> bool {
        self.msg.status_line.code.is_final()
    }
}

/// Transport Layer for SIP messages.
pub struct TransportLayer {
    transports: Mutex<HashMap<TransportKey, Transport>>,

    factorys: Mutex<Vec<Box<dyn Factory>>>,

    // Channel to send packets to the transport layer.
    transport_tx: TransportTx,
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

        Self {
            transport_tx,
            transports: Mutex::new(HashMap::new()),
            factorys: Mutex::new(Vec::new()),
            transport_rx: Mutex::new(Some(transport_rx)),
        }
    }

    pub(crate) fn transport_count(&self) -> usize {
        self.transports.lock().expect("Lock failed").len()
    }

    pub(crate) fn add_transport(&self, transport: Transport) {
        self.transports
            .lock()
            .expect("Lock failed")
            .insert(transport.key(), transport);
    }

    pub(crate) fn remove_transport(&self, key: TransportKey) -> Option<Transport> {
        self.transports.lock().expect("Lock failed").remove(&key)
    }

    pub(crate) fn add_factory(&self, factory: Box<dyn Factory>) {
        self.factorys.lock().expect("Lock failed").push(factory);
    }

    pub(crate) fn sender(&self) -> &TransportTx {
        &self.transport_tx
    }

    /// Finds a suitable transport for the given destination address and transport type.
    pub fn find(&self, dst: SocketAddr, transport: TransportKind) -> Option<Transport> {
        log::debug!("Finding suitable transport={} for={}", transport, dst);

        let transports = self.transports.lock().expect("Lock failed");

        // find by remote addr
        let key = TransportKey::new(dst, transport);

        if let Some(transport) = transports.get(&key) {
            return Some(transport.clone());
        };

        // Find by transport protocol and address family
        // TODO: create transport if tcp or tls(find factory)
        transports
            .values()
            .filter(|handle| handle.tp_kind() == transport && handle.is_same_address_family(&dst))
            .min_by(|a, b| a.cmp(b))
            .cloned()
    }

    pub(crate) async fn receive_packet(&self, endpoint: &Endpoint) -> Result<()> {
        let mut rx = self.transport_rx.lock().expect("Lock failed").take().unwrap();

        // Loop to receive packets from the transports.
        while let Some(evt) = rx.recv().await {
            match evt {
                TransportEvent::PacketReceived(msg) => {
                    tokio::spawn(Self::on_received_packet(msg, endpoint.clone()));
                }
                TransportEvent::TransportCreated(transport) => {
                    self.add_transport(transport);
                }
                TransportEvent::TransportClosed(key) => {
                    self.remove_transport(key);
                }
                TransportEvent::FactoryCreated(factory) => {
                    self.add_factory(factory);
                }
            }
        }

        Ok(())
    }

    async fn on_received_packet(pkt: TransportPacket, endpoint: Endpoint) -> Result<()> {
        let (transport, packet) = pkt;
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
        let mut parser = ParseCtx::new(bytes);
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
        let mut exists_from = false;
        let mut call_id: Option<CallId> = None;
        let mut exists_to = false;

        for header in msg.headers().iter() {
            match header {
                Header::From(_) => exists_from = true,
                Header::To(_) => exists_to = true,
                Header::CallId(c) => call_id = Some(*c),
                Header::CSeq(c) => cseq = Some(*c),
                _ => (),
            }
        }

        if !exists_from {
            return Err(Error::MissingRequiredHeader(FromHdr::NAME));
        }

        if !exists_to {
            return Err(Error::MissingRequiredHeader(To::NAME));
        }

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

        match msg {
            SipMsg::Request(msg) => {
                let mut request = IncomingRequest {
                    msg,
                    transport,
                    packet,
                    tsx: None,
                    req_headers: RequestHeaders { via, cseq, call_id },
                };
                endpoint.process_request(&mut request).await?;
            }
            SipMsg::Response(msg) => {
                let mut response = IncomingResponse { msg, transport, packet };
                endpoint.process_response(&mut response).await?;
            }
        };

        Ok(())
    }
}

#[async_trait::async_trait]
/// A trait to start a new transport.
pub(crate) trait TransportStartup {
    async fn start(&self, tx: TransportTx) -> Result<()>;
}

#[derive(Clone)]
/// This type represents a concret SIP transport implementation.
pub struct Transport {
    inner: Arc<dyn SipTransport>,
}

impl Transport {
    /// Creates a new `Transport` instance with the given transport implementation.
    pub fn new(transport: impl SipTransport) -> Self {
        Self {
            inner: Arc::new(transport),
        }
    }
}

impl PartialOrd for Transport {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transport {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::strong_count(&self.inner).cmp(&Arc::strong_count(&other.inner))
    }
}

impl Eq for Transport {}

impl PartialEq for Transport {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}

impl Deref for Transport {
    type Target = dyn SipTransport;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl From<UdpTransport> for Transport {
    fn from(value: UdpTransport) -> Self {
        Self { inner: Arc::new(value) }
    }
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
            let kind = TransportKind::Udp;

            transports.add_transport(Transport::new(MockUdpTransport));

            assert!(transports.find(addr, kind).is_some());
            assert!(transports.transport_count() == 1);
        }

        #[test]
        fn test_remove_transport() {
            let transports = TransportLayer::default();
            let udp_tp = Transport::new(MockUdpTransport);
            let addr = "127.0.0.1:8080".parse().unwrap();
            let kind = TransportKind::Udp;
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
