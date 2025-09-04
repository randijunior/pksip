#![warn(missing_docs)]
//! SIP Transport Layer.
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::SystemTime;

use bytes::BufMut;
use bytes::Bytes;
use bytes::BytesMut;
use tokio::sync::mpsc;

use crate::core::SipEndpoint;
use crate::error::Error;
use crate::error::Result;
use crate::header::CSeq;
use crate::header::CallId;
use crate::header::ContentLength;
use crate::header::From as FromHdr;
use crate::header::Header;
use crate::header::HeaderParser;
use crate::header::Headers;
use crate::header::To;
use crate::header::Via;
use crate::header::{self};
use crate::message::HostPort;
use crate::message::Request;
use crate::message::Response;
use crate::message::Scheme;
use crate::message::SipMessage;
use crate::message::SipMethod;
use crate::message::StatusCode;
use crate::parser::Parser;
use crate::transaction::key::TransactionKey;
use crate::transaction::ClientTsx;
use crate::transaction::ServerTsx;

mod decoder;

pub mod tcp;
pub mod udp;
pub mod ws;

/// Represents a reference-counted handle to a transport
/// implementation.
pub type TransportRef = Arc<dyn Transport>;

/// This trait represents a abstraction over a SIP transport
/// implementation.
#[async_trait::async_trait]
pub trait Transport: Send + Sync + 'static {
    /// Sends a buffer to the specified remote socket
    /// address.
    ///
    /// Returns the number of bytes sent or an I/O error.
    async fn send(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize>;

    /// Returns the transport protocol (e.g., UDP, TCP,
    /// TLS).
    fn protocol(&self) -> TransportType;

    /// Returns the local socket address bound to this
    /// transport.
    fn addr(&self) -> SocketAddr;

    /// Checks if the provided address belongs to the same
    /// IP address family (IPv4 vs IPv6) as the local
    /// socket address.
    fn is_same_af(&self, addr: &SocketAddr) -> bool {
        let our_addr = self.addr();

        (addr.is_ipv4() && our_addr.is_ipv4()) || (addr.is_ipv6() && our_addr.is_ipv6())
    }

    /// Returns the local transport name.
    fn local_name(&self) -> Cow<'_, str>;

    /// Returns `true` if the transport is reliable (e.g.,
    /// TCP or TLS).
    fn reliable(&self) -> bool;

    /// Returns `true` if the transport is secure (e.g.,
    /// TLS).
    fn secure(&self) -> bool;

    /// Returns the key that uniquely identifies this
    /// transport connection.
    fn key(&self) -> TransportKey {
        TransportKey::new(self.addr(), self.protocol())
    }
}

const UDP: &str = "UDP";
const TCP: &str = "TCP";
const TLS: &str = "TLS";
const SCTP: &str = "SCTP";
const WS: &str = "WS";
const WSS: &str = "WSS";
const UNKNOWN: &str = "UNKNOWN";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
/// An SIP Transport Protocol.
pub enum TransportType {
    #[default]
    /// `UDP` transport.
    Udp,
    /// `TCP` transport.
    Tcp,
    /// `WebSocket` transport.
    Ws,
    /// `WebSocket Secure` transport.
    Wss,
    /// `TLS` transport.
    Tls,
    /// `SCTP` transport.
    Sctp,
    /// UNKNOW transport.
    Unknown,
}

impl TransportType {
    /// Returns the default port number associated with the
    /// transport protocol.
    ///
    /// - `UDP`, `TCP`, and `SCTP` use port `5060` by default.
    /// - `TLS` uses port `5061`.
    /// - `WS` uses port `80`.
    /// - `Unknown` returns `0` to indicate no default.
    #[inline]
    pub const fn get_port(&self) -> u16 {
        match self {
            TransportType::Udp | TransportType::Tcp | TransportType::Sctp => 5060,
            TransportType::Tls => 5061,
            TransportType::Ws | TransportType::Wss => 80,
            TransportType::Unknown => 0,
        }
    }
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Udp => f.write_str(UDP),
            TransportType::Tcp => f.write_str(TCP),
            TransportType::Ws => f.write_str(WS),
            TransportType::Wss => f.write_str(WSS),
            TransportType::Tls => f.write_str(TLS),
            TransportType::Sctp => f.write_str(SCTP),
            TransportType::Unknown => f.write_str(UNKNOWN),
        }
    }
}

impl TransportType {
    /// Returns the transport string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportType::Udp => UDP,
            TransportType::Tcp => TCP,
            TransportType::Ws => WS,
            TransportType::Wss => WSS,
            TransportType::Tls => TLS,
            TransportType::Sctp => SCTP,
            TransportType::Unknown => UNKNOWN,
        }
    }
}

impl From<&str> for TransportType {
    fn from(s: &str) -> Self {
        s.as_bytes().into()
    }
}

impl From<&[u8]> for TransportType {
    fn from(b: &[u8]) -> Self {
        match b {
            b"UDP" | b"udp" => TransportType::Udp,
            b"TCP" | b"tcp" => TransportType::Tcp,
            b"WS" | b"ws" => TransportType::Ws,
            b"WSS" | b"wss" => TransportType::Wss,
            b"TLS" | b"tls" => TransportType::Tls,
            b"SCTP" | b"sctp" => TransportType::Sctp,
            _ => TransportType::Unknown,
        }
    }
}

/// This type represents a key used to identify a transport
/// connection.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TransportKey {
    /// The socket address of the transport.
    addr: SocketAddr,
    /// The transport kind (e.g., UDP, TCP, TLS).
    kind: TransportType,
}

impl TransportKey {
    /// Creates a new `TransportKey`.
    pub fn new(addr: SocketAddr, kind: TransportType) -> Self {
        TransportKey { addr, kind }
    }
}

/// This trait represents a factory for creating SIP
/// transports.
///
/// Normally, this is used by connection oriented transports
/// like TCP and TLS.
#[async_trait::async_trait]
pub trait Factory: Sync + Send + 'static {
    /// Creates a new transport instance.
    async fn create(&self, addr: SocketAddr) -> Result<TransportRef>;

    /// Returns the transport protocol this factory creates.
    fn protocol(&self) -> TransportType;
}

#[derive(Clone, Copy)]
enum Direction {
    Outgoing,
    Incoming,
}

/// Represents the raw binary content of a message or data
/// block.
///
/// Commonly used for message bodies, network packets, or
/// media content.
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
        protocol: TransportType,
    },
    /// SocketAddr address.
    Addr {
        /// The socket address.
        addr: SocketAddr,
        /// The transport to use.
        transport: TransportRef,
    },
}

/// This trait is used to convert a type into a byte buffer.
pub trait Encode: Sized {
    /// Converts the type into a byte buffer.
    fn encode(&self) -> Result<Bytes>;
}

/// This type represents an outgoing SIP response.
pub struct OutgoingResponse {
    /// The SIP response message.
    pub response: Response,
    /// The address to send the response to.
    pub addr: OutgoingAddr,
    /// The message raw buffer.
    pub buf: Option<Bytes>,
}

impl OutgoingResponse {
    /// Returns the message status code.
    pub fn status_code(&self) -> StatusCode {
        self.response.status_line.code
    }

    /// Append headers to the message.
    pub fn append_headers(&mut self, other: &mut Headers) {
        self.response.append_headers(other);
    }

    /// Returns the message rason text.
    pub fn reason(&self) -> &str {
        &self.response.status_line.reason
    }

    /// Returns `true` if this is a provisional response.
    pub fn is_provisional(&self) -> bool {
        self.response.status_line.code.is_provisional()
    }

    /// Set the message body.
    pub fn set_body(&mut self, body: &[u8]) {
        self.response.body = Some(body.into());
    }

    /// Returns a mutable reference to the headers of the
    /// message.
    pub fn headers_mut(&mut self) -> &mut Headers {
        &mut self.response.headers
    }
}

impl Encode for OutgoingResponse {
    fn encode(&self) -> Result<Bytes> {
        let estimated_message_size = if self.response.body.is_none() {
            800
        } else {
            1500
        };
        let buf = BytesMut::with_capacity(estimated_message_size);

        let mut buf_writer = buf.writer();

        // Status Line.
        write!(buf_writer, "{}", &self.response.status_line)?;

        // Headers.
        for header in self.response.headers.iter() {
            write!(buf_writer, "{header}\r\n")?;
        }

        // Body.
        if let Some(body) = &self.response.body {
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
pub struct OutgoingRequest {
    /// The SIP request message.
    pub msg: Request,
    /// The addr to send the request to.
    pub addr: SocketAddr,
    /// The message raw buffer.
    pub buf: Option<Bytes>,
    /// The transport to use for sending the request.
    pub transport: TransportRef,
}

impl Encode for OutgoingRequest {
    fn encode(&self) -> Result<Bytes> {
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
        if let Some(body) = &self.msg.body {
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

/// This type represents mandatory request headers.
pub(crate) struct RequiredHeaders {
    // The topmost Via header as found in the message.
    pub via: Via,
    // The From header as found in the message.
    pub from: header::From,
    // The CSeq header as found in the message.
    pub cseq: CSeq,
    // The Call-ID header found in the message.
    pub call_id: CallId,
    /// The To header as found in the message.
    pub to: To,
}

/// This type represents an received SIP request.
pub struct IncomingRequest {
    /// The SIP request message.
    pub msg: Request,
    /// The request headers extracted from the request.
    pub(crate) request_headers: RequiredHeaders,
    /// The transport used to receive the request.
    pub(crate) transport: TransportRef,
    /// The packet that contains the request.
    pub(crate) packet: Packet,
    /// The server transaction associated with this request,
    /// if any.
    pub(crate) transaction: Option<ServerTsx>,
}

impl IncomingRequest {
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

    /// Returns the transaction key for this request(if
    /// any).
    pub fn tsx_key(&self) -> Option<&TransactionKey> {
        self.transaction.as_ref().map(|tsx| tsx.key())
    }

    /// Returns `true` if the message method matches the
    /// given `SipMethod`.
    #[inline(always)]
    pub fn is_method(&self, method: &SipMethod) -> bool {
        self.msg.method().eq(method)
    }

    /// Returns the message method.
    pub fn method(&self) -> SipMethod {
        self.msg.method()
    }

    /// Gets the source socket address of the packet.
    pub fn addr(&self) -> &SocketAddr {
        &self.packet.addr
    }

    /// Returns `true` if this message was received over a
    /// secure transport (such as TLS) and the request
    /// URI uses the `sips` scheme.
    ///
    /// According to RFC 3261, a SIP message is considered
    /// secure if it was transmitted over a secure
    /// transport (e.g., TLS) and uses the `sips:`
    /// URI scheme, indicating that every hop along the
    /// request path must be secure.
    pub fn is_secure(&self) -> bool {
        self.transport.secure() && self.msg.req_line.uri.scheme == Scheme::Sips
    }
}

/// Represents an received SIP response.
pub struct IncomingResponse {
    /// The SIP response message.
    pub(crate) response: Response,
    /// The transport used to receive the response.
    pub(crate) transport: TransportRef,
    /// The packet that contains the response.
    pub(crate) packet: Packet,
    /// The transaction associated with this response, if
    /// any.
    pub(crate) transaction: Option<ClientTsx>,
    /// The request headers extracted from the response.
    pub(crate) request_headers: RequiredHeaders,
}

/// Represents a message exchanged between the transport
/// layer and other components of the SIP stack. This enum
/// encapsulates different types of events related to
/// transport operations.
pub(crate) enum TransportMessage {
    /// A packet was received from the transport layer.
    ///
    /// Contains the transport that received the packet and
    /// the packet itself.
    Packet {
        transport: TransportRef,
        packet: Packet,
    },
    /// A new transport was created.
    Created(TransportRef),
    /// A transport was closed.
    Closed(TransportKey),
}

type TransportTx = mpsc::Sender<TransportMessage>;
type TransportRx = mpsc::Receiver<TransportMessage>;

/// Transport Layer for SIP messages.
pub struct Transports {
    /// A map of transports indexed by their unique keys.
    transports: Mutex<HashMap<TransportKey, TransportRef>>,
    /// A list of transport factories.
    factories: Vec<Box<dyn Factory>>,
    /// The transport sender used to send events to the
    /// transport layer.
    tp_tx: TransportTx,
    /// A receiver for transport events.
    tp_rx: Mutex<Option<TransportRx>>,
}

impl Transports {
    pub(crate) fn new(factories: Vec<Box<dyn Factory>>) -> Self {
        let (tp_tx, tp_rx) = mpsc::channel(1_000);
        let tp_rx = Mutex::new(Some(tp_rx));

        Self {
            tp_tx,
            tp_rx,
            factories,
            transports: Default::default(),
        }
    }

    pub(crate) fn insert(&self, transport: TransportRef) {
        let mut map = self.transports.lock().expect("Lock failed");

        map.insert(transport.key(), transport);
    }

    pub(crate) fn remove(&self, key: TransportKey) -> Option<TransportRef> {
        let mut map = self.transports.lock().expect("Lock failed");

        map.remove(&key)
    }

    pub(crate) fn transport_count(&self) -> usize {
        let map = self.transports.lock().expect("Lock failed");

        map.len()
    }

    pub(crate) fn sender(&self) -> &TransportTx {
        &self.tp_tx
    }

    /// Finds a suitable transport for the given destination
    /// address and transport type.
    pub fn find(&self, dst: SocketAddr, transport: TransportType) -> Option<TransportRef> {
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
            .filter(|handle| handle.protocol() == transport && handle.is_same_af(&dst))
            .min_by(|a, b| Arc::strong_count(a).cmp(&Arc::strong_count(b)))
            .cloned()
    }

    pub(crate) async fn handle_events(&self, endpoint: &SipEndpoint) -> Result<()> {
        let mut rx = self.tp_rx.lock().expect("Lock failed").take().unwrap();

        // Loop to receive packets from the transports.
        while let Some(evt) = rx.recv().await {
            match evt {
                TransportMessage::Packet { transport, packet } => {
                    tokio::spawn(Self::on_received_packet(
                        transport,
                        packet,
                        endpoint.clone(),
                    ));
                }
                TransportMessage::Created(transport) => {
                    self.insert(transport);
                }
                TransportMessage::Closed(key) => {
                    self.remove(key);
                }
            }
        }

        Ok(())
    }

    async fn on_received_packet(
        transport: TransportRef,
        packet: Packet,
        endpoint: SipEndpoint,
    ) -> Result<()> {
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
        let msg = match Parser::parse_sip_msg(bytes) {
            Ok(parsed_msg) => parsed_msg,
            Err(err) => {
                log::warn!(
                    "Ignoring {} bytes packet from {} {} : {}\n{}-- end of packet.",
                    bytes.len(),
                    transport.protocol(),
                    packet.addr,
                    err,
                    String::from_utf8_lossy(bytes)
                );

                return Err(err);
            }
        };

        let headers = msg.headers();
        // Check for mandatory headers.
        let via = crate::find_map_header!(headers, Via).cloned();

        let Some(mut via) = via else {
            return Err(Error::MissingRequiredHeader(Via::NAME));
        };

        let mut cseq: Option<CSeq> = None;
        let mut from: Option<header::From> = None;
        let mut call_id: Option<CallId> = None;
        let mut to: Option<To> = None;

        for header in headers.iter() {
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

        let request_headers = RequiredHeaders {
            via,
            cseq,
            call_id,
            from,
            to,
        };
        match msg {
            SipMessage::Request(msg) => {
                let mut request = Some(IncomingRequest {
                    msg,
                    transport,
                    packet,
                    transaction: None,
                    request_headers,
                });
                endpoint.process_request(&mut request).await?;
            }
            SipMessage::Response(response) => {
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
        // use super::*;
        // use crate::transport::udp::mock::MockUdpTransport;

        // #[test]
        // fn test_add_transport() {
        //     let transports = Transports::default();
        //     let addr = "127.0.0.1:8080".parse().unwrap();
        //     let kind = TransportType::Udp;

        //     transports.
        // insert(Arc::new(MockUdpTransport));

        //     assert!(transports.find(addr,
        // kind).is_some());     assert!(transports.
        // transport_count() == 1); }

        // #[test]
        // fn test_remove_transport() {
        //     let transports = Transports::default();
        //     let udp_tp = Arc::new(MockUdpTransport);
        //     let addr = "127.0.0.1:8080".parse().unwrap();
        //     let kind = TransportType::Udp;
        //     let key = udp_tp.key();

        //     transports.insert(udp_tp);
        //     assert!(transports.find(addr,
        // kind).is_some());     assert!(transports.
        // transport_count() == 1);

        //     transports.remove(key);
        //     assert!(transports.find(addr,
        // kind).is_none());     assert!(transports.
        // transport_count() == 0); }
    }
}
