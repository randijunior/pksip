//! Transport layer.
//!
//! This module defines different transport protocols used to
//! exchange SIP messages between entities.
//!
//! All transports implement the [`SipTransport`] trait and expose a
//! common interface for sending and receiving SIP messages.
//!
//! The [`Transport`] struct is a wrapper around any transport implementation
//! that allows for easy management of different transport types.
//!
//! # Available Transports
//!
//! - [`udp`]: SIP over UDP transport implementation.
//! - [`tcp`]: SIP over TCP transport implementation.
//! - [`ws`]:  SIP over WebSocket transport implementation.

use std::{
    collections::HashMap,
    fmt::{self, Formatter, Result as FmtResult},
    io::{self, Write},
    net::{IpAddr, SocketAddr},
    ops::Deref,
    result::Result as StdResult,
    str::FromStr,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use async_trait::async_trait;
use bytes::{BufMut, Bytes, BytesMut};
use tokio::net::ToSocketAddrs;
use util::{NAPTR, RData, SRV};

use crate::{
    Endpoint, SipMethod,
    error::{Error, Result},
    headers::{
        CSeq, CallId, ContentLength, From as FromHdr, Header, HeaderParser, Headers, To, Via,
    },
    message::{
        DomainName, Host, HostPort, ReasonPhrase, Request, Response, Scheme, SipMessage,
        SipMessageBody, SipUri, StatusCode, StatusLine, Uri,
    },
    parser::Parser,
    transaction::{ClientTx, ServerTx},
    transport::udp::UdpTransport,
};

// Core Transport modules
mod decode;
pub mod tcp;
pub mod udp;
pub mod websocket;

/// Keep-alive Request.
pub const KEEPALIVE_REQUEST: &[u8] = b"\r\n\r\n";

/// Keep-alive Response.
pub const KEEPALIVE_RESPONSE: &[u8] = b"\r\n";

/// Marks the end of headers in a SIP message.
pub const MSG_HEADERS_END: &[u8] = b"\r\n\r\n";

/// Type alias for a map of transports.
type TransportsMap = Mutex<HashMap<TransportKey, Transport>>;

/// This type represents an received SIP request.
pub type IncomingRequest = Incoming<Request>;
/// This type represents an received SIP response.
pub type IncomingResponse = Incoming<Response>;

/// This type represents an outbound SIP request.
pub type OutgoingRequest = Outgoing<Request>;
/// This type represents an outgoing SIP response.
pub type OutgoingResponse = Outgoing<Response>;

/// This type is a wrapper around a SIP transport implementation.
#[derive(Clone)]
pub struct Transport {
    /// Shared transport instance.
    shared: Arc<dyn SipTransport>,
}

impl Transport {
    /// Creates a new `Transport` instance with the given implementation.
    pub fn new(transport: impl SipTransport) -> Self {
        Transport {
            shared: Arc::new(transport),
        }
    }
}

impl Deref for Transport {
    type Target = dyn SipTransport;

    fn deref(&self) -> &Self::Target {
        &*self.shared
    }
}

/// Manager for SIP all transports.
pub struct TransportManager {
    /// All transports indexed by their unique keys.
    transports: TransportsMap,
}

impl TransportManager {
    /// Create a new `TransportManager` instance.
    pub fn new() -> Self {
        TransportManager {
            transports: Mutex::new(HashMap::new()),
        }
    }

    /// Add a new transport to the manager.
    pub fn register_transport(&self, transport: Transport) -> Result<()> {
        let key = transport.key();
        let mut map = self.transports.lock().map_err(|_| Error::PoisonedLock)?;

        map.insert(key, transport);

        Ok(())
    }

    /// Remove a transport by its key.
    pub fn remove_transport(&self, key: &TransportKey) -> Result<()> {
        let mut map = self.transports.lock().map_err(|_| Error::PoisonedLock)?;

        map.remove(key);

        Ok(())
    }

    /// Select a suitable transport for the given `Uri`.
    pub async fn select_transport(
        &self,
        endpoint: &Endpoint,
        uri: &Uri,
    ) -> Result<(Transport, SocketAddr)> {
        let target = uri.maddr_param.as_ref().unwrap_or(&uri.host_port.host);
        let port = uri.host_port.port;

        match uri.transport_param {
            Some(transport) => {
                // 1. If transport parameter is specified it takes precedence.
                let port = port.unwrap_or(transport.default_port());
                let ip = endpoint.lookup_address(target).await?;
                let addr = SocketAddr::new(ip, port);
                let transport = self.get_or_create_transport(transport, addr).await?;
                Ok((transport, addr))
            }
            None => match target {
                Host::IpAddr(ip_addr) => {
                    // 2. If no transport parameter and target is an IP address then sip should use
                    // udp and sips tcp.
                    let transport = TransportType::from_scheme(uri.scheme);
                    let port = port.unwrap_or(transport.default_port());
                    let addr = SocketAddr::new(*ip_addr, port);
                    let transport = self.get_or_create_transport(transport, addr).await?;
                    return Ok((transport, addr));
                }
                Host::DomainName(domain) => {
                    if let Some(port) = port {
                        // 3. If no transport parameter and target is a host name with an explicit port
                        // then sip should use udp and sips tcp and host should be resolved using an A
                        // or AAAA record DNS lookup (section 4.2)
                        let transport = TransportType::from_scheme(uri.scheme);
                        let ip = endpoint.dns_lookup(domain).await?;
                        let addr = SocketAddr::new(ip, port);
                        let transport = self.get_or_create_transport(transport, addr).await?;
                        Ok((transport, addr))
                    } else {
                        // 4. If no transport protocol and no explicit port and target is a host name then
                        // the client should do an NAPTR lookup.
                        if let Some((transport, addr)) =
                            self.perform_natptr_query(endpoint, domain).await?
                        {
                            return Ok((transport, addr));
                        } else {
                            todo!()
                        }
                    }
                }
            },
        }
    }
    /// Implements RFC 3263 §4.1 and §4.2
    async fn perform_natptr_query(
        &self,
        endpoint: &Endpoint,
        target: &DomainName,
    ) -> Result<Option<(Transport, SocketAddr)>> {
        let lookup = endpoint.dns_resolver().naptr_query(target.as_str()).await?;
        let naptr_records: Vec<&NAPTR> = lookup
            .record_iter()
            .filter_map(|record| match record.data() {
                RData::NAPTR(naptr) => Some(naptr),
                _record_data => None,
            })
            .collect();
        if naptr_records.is_empty() {
            return Ok(None);
        }
        for record in naptr_records {
            // If NAPTR record(s) are found select the desired transport and lookup the SRV record.
            let Some(transport) = TransportType::from_naptr_service(record.services()) else {
                continue;
            };
            match record.flags() {
                b"s" => {
                    let srv_records = endpoint
                        .dns_resolver()
                        .srv_query(record.replacement().clone())
                        .await?;
                    let srv_records: Vec<&SRV> = srv_records
                        .record_iter()
                        .filter_map(|record| match record.data() {
                            RData::SRV(srv) => Some(srv),
                            _ => None,
                        })
                        .collect();

                    for record in srv_records {
                        let port = record.port();
                        let target = record.target();
                        let lookup = endpoint
                            .dns_resolver()
                            .lookup_ip(target.clone())
                            .await
                            .map_err(|err| {
                                io::Error::other(format!("Failed to lookup DNS: {}", err))
                            })?;
                        for ip in lookup {
                            let addr = SocketAddr::new(ip, port);
                            match self.get_or_create_transport(transport, addr).await {
                                Ok(transport) => return Ok(Some((transport, addr))),
                                Err(_) => continue,
                            }
                        }
                    }

                    return Ok(None);
                }
                b"a" => todo!("resolve_a_records"),
                _ => todo!(""),
            }
        }

        Ok(None)
    }

    async fn get_or_create_transport(
        &self,
        transport_type: TransportType,
        addr: SocketAddr,
    ) -> Result<Transport> {
        let key = TransportKey::new(addr, transport_type);

        let map = self.transports.lock().map_err(|_| Error::PoisonedLock)?;

        if let Some(transport) = map.get(&key) {
            return Ok(transport.clone());
        }
        let transport = map.values().find(|transport| {
            transport.transport_type() == transport_type
                && is_same_ip_family(&transport.local_addr().ip(), &addr.ip())
        });

        if let Some(transport) = transport {
            return Ok(transport.clone());
        } else {
            todo!("create")
        }
    }

    /// Return the number of transports registered.
    pub fn transport_count(&self) -> Result<usize> {
        let map = self.transports.lock().map_err(|_| Error::PoisonedLock)?;

        Ok(map.len())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Represents the type of transport.
pub enum TransportType {
    /// Udp.
    Udp,
    /// Tcp.
    Tcp,
    /// WebSocket.
    Ws,
    /// Websocket with tls.
    Wss,
    /// Tcp with tls
    Tls,
    /// Sctp.
    Sctp,
}

impl TransportType {
    /// Returns true if the transport is reliable.
    pub fn is_reliable(self) -> bool {
        matches!(
            self,
            Self::Tcp | Self::Tls | Self::Sctp | Self::Wss | Self::Ws
        )
    }

    pub fn from_naptr_service(service: &[u8]) -> Option<Self> {
        match service {
            b"SIP+D2U" => Some(Self::Udp),
            b"SIP+D2T" => Some(Self::Tcp),
            b"SIPS+D2T" => Some(Self::Tls),
            _ => None,
        }
    }
    pub fn from_scheme(scheme: Scheme) -> Self {
        match scheme {
            Scheme::Sip => Self::Udp,
            Scheme::Sips => Self::Tcp,
        }
    }

    /// Returns true if the transport is secure.
    pub fn is_secure(self) -> bool {
        matches!(self, Self::Tls | Self::Wss)
    }

    /// Returns the default port number associated with the transport.
    #[inline]
    pub const fn default_port(&self) -> u16 {
        match self {
            Self::Udp | Self::Tcp | Self::Sctp => 5060,
            Self::Tls => 5061,
            Self::Ws | Self::Wss => 80,
        }
    }
}

impl FromStr for TransportType {
    type Err = ();

    fn from_str(s: &str) -> StdResult<Self, Self::Err> {
        match s {
            s if s.eq_ignore_ascii_case("udp") => Ok(Self::Udp),
            s if s.eq_ignore_ascii_case("tcp") => Ok(Self::Tcp),
            s if s.eq_ignore_ascii_case("ws") => Ok(Self::Ws),
            s if s.eq_ignore_ascii_case("wss") => Ok(Self::Wss),
            s if s.eq_ignore_ascii_case("tls") => Ok(Self::Tls),
            s if s.eq_ignore_ascii_case("sctp") => Ok(Self::Sctp),
            _ => Err(()),
        }
    }
}

impl TryFrom<&str> for TransportType {
    type Error = ();

    fn try_from(s: &str) -> StdResult<Self, Self::Error> {
        Self::from_str(s)
    }
}

impl TryFrom<String> for TransportType {
    type Error = ();

    fn try_from(s: String) -> StdResult<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        use self::TransportType::*;

        f.write_str(match self {
            Udp => "UDP",
            Tcp => "TCP",
            Tls => "TLS",
            Sctp => "SCTP",
            Ws => "WS",
            Wss => "WSS",
        })
    }
}

/// Trait for all transport implementations.
#[async_trait]
pub trait SipTransport: Send + Sync + 'static {
    /// Sends data on the socket to the given address. On success, returns the
    /// number of bytes written.
    async fn send_msg(&self, buf: &[u8], address: &SocketAddr) -> Result<usize>;

    /// Get transport type.
    fn transport_type(&self) -> TransportType;

    /// Get the local socket address addr to this transport.
    fn local_addr(&self) -> SocketAddr;

    /// Get the remote socket address addr to this transport (if any).
    fn remote_addr(&self) -> Option<SocketAddr>;

    /// Returns `true` if the transport is reliable.
    fn is_reliable(&self) -> bool {
        self.transport_type().is_reliable()
    }

    /// Returns `true` if the transport is secure.
    fn is_secure(&self) -> bool {
        self.transport_type().is_secure()
    }

    /// Get the id that uniquely identifies this transport.
    fn key(&self) -> TransportKey {
        TransportKey::from(self)
    }
}

/// A factory creating transport instances.
#[async_trait]
pub trait TransportFactory: Sync + Send {
    /// Creates a new transport instance for the given `Endpoint`.
    async fn create<A>(&self, addr: A, endpoint: &Endpoint) -> Result<Transport>
    where
        A: ToSocketAddrs + Send;

    /// Transport protocol that this factory creates.
    fn transport_type(&self) -> TransportType;
}

/// Unique key for a transport instance.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TransportKey {
    /// The destination address of the transport.
    pub address: SocketAddr,
    /// The transport type (e.g., UDP, TCP, TLS).
    pub tp_type: TransportType,
}

impl TransportKey {
    /// Creates a new transport key.
    pub fn new(address: SocketAddr, tp_type: TransportType) -> Self {
        TransportKey { address, tp_type }
    }
}

impl<T> From<&T> for TransportKey
where
    T: SipTransport + ?Sized,
{
    fn from(transport: &T) -> Self {
        let address = transport.local_addr();
        let tp_type = transport.transport_type();

        Self { address, tp_type }
    }
}

/// A raw network packet.
pub struct Packet {
    /// Raw packet payload.
    pub data: Bytes,
    /// Remote address of the sender.
    pub source: SocketAddr,
    /// Time when the packet was received.
    pub timestamp: SystemTime,
}

impl Packet {
    /// Creates a new `Packet` whith the given `data` and `source` addr.
    pub fn new(data: Bytes, source: SocketAddr) -> Self {
        Self {
            data,
            source,
            timestamp: SystemTime::now(),
        }
    }
}

/// A network packet received through a transport.
pub struct TransportMessage {
    /// Transport that received the packet.
    pub transport: Transport,
    /// The raw packet data and metadata.
    pub packet: Packet,
}

impl TransportMessage {
    /// Parse the packet into an sip message.
    pub fn parse(self) -> Result<IncomingMessage> {
        let Self { transport, packet } = &self;
        let sip_message = match Parser::parse(&packet.data) {
            Ok(parsed) => parsed,
            Err(err) => {
                log::warn!(
                    "Ignoring {} bytes packet from {} {} : {}\n{}-- end of packet.",
                    packet.data.len(),
                    transport.transport_type(),
                    packet.source,
                    err,
                    String::from_utf8_lossy(&packet.data)
                );

                return Err(err);
            }
        };
        // Check for mandatory headers.
        let mut headers: MandatoryHeaders = sip_message.headers().try_into()?;
        // 4. Server Behavior
        // the server MUST insert a "received" parameter containing the source
        // IP address that the request came from.
        headers.via.received = packet.source.ip().into();

        let info = IncomingMessageInfo::new(self, headers);
        let message = match sip_message {
            SipMessage::Request(request) => {
                let request = IncomingRequest::new(request, info);
                IncomingMessage::Request(request)
            }
            SipMessage::Response(response) => {
                let response = IncomingResponse::new(response, info);
                IncomingMessage::Response(response)
            }
        };

        Ok(message)
    }
}

/// An incoming SIP message delivered by a transport.
pub enum IncomingMessage {
    /// A SIP request.
    Request(IncomingRequest),
    /// A SIP response.
    Response(IncomingResponse),
}

impl IncomingMessage {
    pub fn msg_info(&self) -> &IncomingMessageInfo {
        match self {
            IncomingMessage::Request(incoming) => &incoming.info,
            IncomingMessage::Response(incoming) => &incoming.info,
        }
    }
}

pub enum OutgoingMessage {
    /// A SIP request.
    Request(OutgoingRequest),
    /// A SIP response.
    Response(OutgoingResponse),
}

impl OutgoingRequest {
    pub fn new(message: Request, send_info: OutgoingMessageInfo) -> Self {
        Self {
            message,
            send_info,
            encoded: Bytes::new(),
        }
    }
}

impl From<OutgoingRequest> for OutgoingMessage {
    fn from(value: OutgoingRequest) -> Self {
        Self::Request(value)
    }
}

impl OutgoingMessage {
    pub fn get_or_insert_encoded(&mut self) -> Result<()> {
        match self {
            OutgoingMessage::Request(outgoing) => {
                if outgoing.encoded.is_empty() {
                    outgoing.encoded = outgoing.encode()?;
                }
            }
            OutgoingMessage::Response(outgoing) => {
                if outgoing.encoded.is_empty() {
                    outgoing.encoded = outgoing.encode()?;
                }
            }
        };

        Ok(())
    }

    pub fn buffer(&self) -> Result<Bytes> {
        match self {
            OutgoingMessage::Request(outgoing) => outgoing.encode(),
            OutgoingMessage::Response(outgoing) => outgoing.encode(),
        }
    }

    pub fn outgoing_info(&self) -> &OutgoingMessageInfo {
        match self {
            OutgoingMessage::Request(outgoing) => &outgoing.send_info,
            OutgoingMessage::Response(outgoing) => &outgoing.send_info,
        }
    }
}

/// Represents the mandatory headers that every SIP message must contain.
pub struct MandatoryHeaders {
    /// The topmost `Via` header.
    pub via: Via,
    /// The `From` header.
    pub from: FromHdr,
    /// The `To` header.
    pub to: To,
    /// The `Call-ID` header.
    pub call_id: CallId,
    /// The `CSeq` header.
    pub cseq: CSeq,
}

impl MandatoryHeaders {
    /// Extracts a mandatory header.
    pub fn required<T>(header: Option<T>, name: &'static str) -> Result<T> {
        header.ok_or(Error::MissingHeader(name))
    }
}

impl TryFrom<&Headers> for MandatoryHeaders {
    type Error = Error;

    fn try_from(headers: &Headers) -> StdResult<Self, Self::Error> {
        let mut via: Option<Via> = None;
        let mut cseq: Option<CSeq> = None;
        let mut from: Option<FromHdr> = None;
        let mut call_id: Option<CallId> = None;
        let mut to: Option<To> = None;

        for header in headers.iter() {
            match header {
                Header::Via(v) if via.is_none() => via = Some(v.clone()),
                Header::From(f) => from = Some(f.clone()),
                Header::To(t) => to = Some(t.clone()),
                Header::CallId(c) => call_id = Some(c.clone()),
                Header::CSeq(c) => cseq = Some(*c),
                _ => (),
            }
        }
        let via = Self::required(via, Via::NAME)?;
        let from = Self::required(from, FromHdr::NAME)?;
        let to = Self::required(to, To::NAME)?;
        let call_id = Self::required(call_id, CallId::NAME)?;
        let cseq = Self::required(cseq, CSeq::NAME)?;

        Ok(MandatoryHeaders {
            via,
            from,
            to,
            call_id,
            cseq,
        })
    }
}

struct TransportDestinationInfo {
    /// Transport and address to use.
    transport: Option<(Transport, SocketAddr)>,
    /// Destination host to contact
    destination_host: Option<HostPort>,
}

/// Outgoing message.
pub struct Outgoing<M> {
    /// The SIP message (request or response).
    pub message: M,
    /// Metadata about how the message will be sent.
    pub send_info: OutgoingMessageInfo,

    /// Message encoded representation.
    pub encoded: Bytes,
}

impl<M> Outgoing<M> {
    pub(crate) fn write_body<W: Write>(
        &self,
        writer: &mut W,
        body: &Option<SipMessageBody>,
    ) -> Result<()> {
        const CONTENT_LENGTH: &str = ContentLength::NAME;
        if let Some(body) = body {
            write!(writer, "{CONTENT_LENGTH}: {}\r\n", body.len())?;
            write!(writer, "\r\n")?;
            writer.write_all(body)?;
        } else {
            write!(writer, "{CONTENT_LENGTH}: 0\r\n")?;
            write!(writer, "\r\n")?;
        }
        Ok(())
    }
}

/// Outgoing message info.
#[derive(Clone)]
pub struct OutgoingMessageInfo {
    /// The socket this message should be sent to.
    pub destination: SocketAddr,
    /// The transport to use for sending the message.
    pub transport: Transport,
}

/// Incoming message.
pub struct Incoming<M> {
    /// The SIP message.
    pub message: M,
    /// Incoming message info.
    pub info: IncomingMessageInfo,
}

impl<M> Incoming<M> {
    /// Returns `true` if this message was received over a secure transport.
    pub fn transport_is_secure(&self) -> bool {
        self.info.received_packet.transport.is_secure()
    }
}

/// Incoming message info.
pub struct IncomingMessageInfo {
    /// The mandatory headers extracted from the message.
    pub mandatory_headers: MandatoryHeaders,
    /// The received transport packet.
    pub received_packet: TransportMessage,
}

impl IncomingMessageInfo {
    /// Creates a new `IncomingMessageInfo`.
    pub fn new(received_packet: TransportMessage, mandatory_headers: MandatoryHeaders) -> Self {
        IncomingMessageInfo {
            mandatory_headers,
            received_packet,
        }
    }
}

impl IncomingRequest {
    /// Creates a new `IncomingRequest`.
    pub fn new(message: Request, info: IncomingMessageInfo) -> Self {
        IncomingRequest { message, info }
    }
    /// Creates a new SIP response based on an incoming
    /// request.
    ///
    /// This method generates a response message with the specified status code
    /// and reason phrase. It also sets the necessary headers from request,
    /// including `Call-ID`, `From`, `To`, `CSeq`, `Via` and
    /// `Record-Route` headers.
    pub fn new_response(
        &self,
        status_code: StatusCode,
        reason_phrase: Option<impl Into<ReasonPhrase>>,
    ) -> OutgoingResponse {
        // Copy the necessary headers from the request.
        let mut headers = Headers::with_capacity(7);
        let all_hdrs = &self.message.headers;
        let req_hdrs = &self.info.mandatory_headers;

        // `Via` header.
        let topmost_via = req_hdrs.via.clone();
        headers.push(Header::Via(topmost_via));
        let other_vias = all_hdrs
            .iter()
            .filter(|h| matches!(h, Header::Via(_)))
            .skip(1);
        headers.extend(other_vias.cloned());

        // `Record-Route` header.
        let rr = all_hdrs
            .iter()
            .filter(|h| matches!(h, Header::RecordRoute(_)));
        headers.extend(rr.cloned());

        // `Call-ID` header.
        headers.push(Header::CallId(req_hdrs.call_id.clone()));

        // `From` header.
        headers.push(Header::From(req_hdrs.from.clone()));

        // `To` header.
        let mut to = req_hdrs.to.clone();
        // 8.2.6.2 Headers and Tags
        // The UAS MUST add a tag to the To header field in
        // the response (with the exception of the 100 (Trying)
        // response, in which a tag MAY be present).
        if to.tag().is_none() && status_code.as_u16() > 100 {
            to.set_tag(req_hdrs.via.branch.clone());
        }
        headers.push(Header::To(to));

        // `CSeq` header.
        headers.push(Header::CSeq(req_hdrs.cseq));

        let reason = match reason_phrase {
            None => status_code.reason(),
            Some(reason) => reason.into(),
        };
        let status_line = StatusLine::new(status_code, reason);

        // Done.
        OutgoingResponse {
            message: Response {
                status_line,
                headers,
                body: None,
            },
            send_info: OutgoingMessageInfo {
                destination: self.info.received_packet.packet.source,
                transport: self.info.received_packet.transport.clone(),
            },
            encoded: Bytes::new(),
        }
    }
}

impl IncomingResponse {
    /// Creates a new `IncomingResponse`.
    pub fn new(message: Response, info: IncomingMessageInfo) -> Self {
        IncomingResponse { message, info }
    }
}

/// Trait for converting a type into into a buffer.
pub trait Encode {
    /// The buffer type that holds the encoded data.
    type Buffer: AsRef<[u8]>;
    /// Converts the type into a byte buffer.
    fn encode(&self) -> Result<Self::Buffer>;
}

impl Encode for OutgoingResponse {
    type Buffer = Bytes;

    fn encode(&self) -> Result<Self::Buffer> {
        let response = &self.message;
        let buf = BytesMut::new();
        let mut writer = buf.writer();

        write!(writer, "{}", response.status_line)?;
        write!(writer, "{}", response.headers)?;
        self.write_body(&mut writer, &response.body)?;

        Ok(writer.into_inner().freeze())
    }
}

impl Encode for OutgoingRequest {
    type Buffer = Bytes;

    fn encode(&self) -> Result<Self::Buffer> {
        let request = &self.message;
        let buf = BytesMut::new();
        let mut writer = buf.writer();

        write!(writer, "{}", request.req_line)?;
        write!(writer, "{}", request.headers)?;
        self.write_body(&mut writer, &request.body)?;

        Ok(writer.into_inner().freeze())
    }
}

fn is_same_ip_family(first: &IpAddr, second: &IpAddr) -> bool {
    match (first, second) {
        (IpAddr::V4(_), IpAddr::V4(_)) => true,
        (IpAddr::V6(_), IpAddr::V6(_)) => true,
        _ => false,
    }
}

async fn resolve_srv_records(transport_type: Option<TransportType>) {}

#[cfg(test)]
pub(crate) mod mock {
    use std::net::Ipv4Addr;

    use tokio::sync::Mutex as AsyncMutex;

    use super::*;
    pub struct MockTransport {
        pub sent: AsyncMutex<Vec<(Vec<u8>, SocketAddr)>>,
        pub addr: SocketAddr,
        pub tp_type: TransportType,
    }

    impl MockTransport {
        pub fn with_transport_type(tp_type: TransportType) -> Transport {
            let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
            let port = tp_type.default_port();
            let mock = Self {
                sent: Default::default(),
                addr: SocketAddr::new(ip, port),
                tp_type,
            };

            Transport::new(mock)
        }

        pub fn new_udp() -> Transport {
            Self::with_transport_type(TransportType::Udp)
        }

        pub fn new_tcp() -> Transport {
            Self::with_transport_type(TransportType::Tcp)
        }

        pub fn new_tls() -> Transport {
            Self::with_transport_type(TransportType::Tls)
        }
    }

    #[async_trait::async_trait]
    impl SipTransport for MockTransport {
        async fn send_msg(&self, buf: &[u8], address: &SocketAddr) -> Result<usize> {
            self.sent.lock().await.push((buf.to_vec(), *address));
            Ok(buf.len())
        }

        fn remote_addr(&self) -> Option<SocketAddr> {
            None
        }

        fn transport_type(&self) -> TransportType {
            self.tp_type
        }

        fn local_addr(&self) -> SocketAddr {
            self.addr
        }
    }
}

#[cfg(test)]
mod tests {
    use mock::MockTransport;

    use super::*;

    #[test]
    fn test_sip_transport() {
        let transport = MockTransport::new_udp();
        assert_eq!(transport.transport_type(), TransportType::Udp);
        assert!(!transport.is_reliable());
        assert!(!transport.is_secure());

        let transport = MockTransport::new_tcp();
        assert_eq!(transport.transport_type(), TransportType::Tcp);
        assert!(transport.is_reliable());
        assert!(!transport.is_secure());

        let transport = MockTransport::new_tls();
        assert_eq!(transport.transport_type(), TransportType::Tls);
        assert!(transport.is_reliable());
        assert!(transport.is_secure());
    }

    #[test]
    fn test_transport_type() {
        let udp = TransportType::Udp;
        assert_eq!(udp.default_port(), 5060);
        assert!(!udp.is_reliable());
        assert!(!udp.is_secure());

        let tcp = TransportType::Tcp;
        assert_eq!(tcp.default_port(), 5060);
        assert!(tcp.is_reliable());
        assert!(!tcp.is_secure());

        let tls = TransportType::Tls;
        assert_eq!(tls.default_port(), 5061);
        assert!(tls.is_reliable());
        assert!(tls.is_secure());

        let ws = TransportType::Ws;
        assert_eq!(ws.default_port(), 80);
        assert!(ws.is_reliable());
        assert!(!ws.is_secure());
    }

    #[test]
    fn test_transport_type_from_string() {
        let tp_type: TransportType = "UDP".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Udp);
        let tp_type: TransportType = "udp".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Udp);

        let tp_type: TransportType = "TCP".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Tcp);
        let tp_type: TransportType = "tcp".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Tcp);

        let tp_type: TransportType = "TLS".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Tls);
        let tp_type: TransportType = "tls".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Tls);

        let tp_type: TransportType = "WS".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Ws);
        let tp_type: TransportType = "ws".try_into().unwrap();
        assert_eq!(tp_type, TransportType::Ws);
    }

    #[test]
    fn test_transport_manager() {
        let manager = TransportManager::new();
        let transport = MockTransport::new_udp();
        let addr = transport.local_addr();
        let tp_type = transport.transport_type();
        let key = transport.key();

        // manager.register_transport(transport).unwrap();
        // assert_eq!(manager.transport_count().unwrap(), 1);

        // let selected = manager.select_transport(addr, TransportType::Udp);
        // let selected = selected.unwrap().unwrap();
        // assert_eq!(selected.transport_type(), tp_type);
        // assert_eq!(selected.local_addr(), addr);

        // manager.remove_transport(&key).unwrap();
        // assert_eq!(manager.transport_count().unwrap(), 0);
    }

    #[test]
    fn test_transport_key_from_tp() {
        let transport = MockTransport::new_udp();
        let key: TransportKey = (transport.deref()).into();
        assert_eq!(key.address, transport.local_addr());
        assert_eq!(key.tp_type, transport.transport_type());
    }
}
