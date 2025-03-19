use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{self},
    net::SocketAddr,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub mod udp;

use async_trait::async_trait;

use std::io::Write;
use tokio::sync::mpsc;
use udp::Udp;

use crate::{
    endpoint::Endpoint,
    headers::{self, CSeq, CallId, SipHeader, To, Via},
    message::{
        HostPort, SipMethod, SipMsg, SipRequest, SipResponse,
        StatusCode, TransportProtocol,
    },
    parser::{self}
};

pub(crate) const CRLF: &[u8] = b"\r\n";
pub(crate) const END: &[u8] = b"\r\n\r\n";
pub(crate) const MAX_PACKET_SIZE: usize = 4000;

pub(crate) type TpSender = mpsc::Sender<(Transport, Packet)>;

#[derive(Debug)]
pub struct MsgBuffer(Vec<u8>);

impl Deref for MsgBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for MsgBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl MsgBuffer {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::with_capacity(MAX_PACKET_SIZE))
    }

    #[inline]
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        self.0.extend_from_slice(other);
    }
}

impl Write for MsgBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ConnectionKey {
    addr: SocketAddr,
    protocol: TransportProtocol,
}

impl ConnectionKey {
    pub fn new(
        addr: SocketAddr,
        protocol: TransportProtocol,
    ) -> Self {
        ConnectionKey { addr, protocol }
    }
}

pub struct TransportLayer {
    transports: Mutex<HashMap<ConnectionKey, Transport>>,
}

impl Default for TransportLayer {
    fn default() -> Self {
        Self {
            transports: Mutex::new(HashMap::new()),
        }
    }
}

impl TransportLayer {
    pub fn new() -> Self {
        Self::default()
    }

    // message_oriented
    // connection_oriented

    //TODO: send
    //TODO: remove
    //TODO: shutdown

    pub fn add(&mut self, transport: Transport) {
        let mut tps = self.transports.lock().unwrap();

        tps.insert(
            transport.key(),
            Transport(Arc::clone(&transport.0)),
        );
    }

    pub fn find(
        &self,
        dst: SocketAddr,
        protocol: TransportProtocol,
    ) -> Option<Transport> {
        println!(
            "Finding suitable transport={} for={}",
            protocol, dst
        );
        let transports = self.transports.lock().unwrap();

        // find by remote addr
        let key = ConnectionKey::new(dst, protocol);
        if let Some(tp) = transports.get(&key) {
            return Some(tp.clone());
        };

        // Find by transport protocol and address family
        // TODO: create transport if tcp or tls
        transports
            .values()
            .filter(|transport| {
                transport.protocol() == protocol
                    && transport.is_same_address_family(&dst)
            })
            .min_by(|a, b| a.cmp(b))
            .cloned()
    }

    pub fn initialize(&self) -> mpsc::Receiver<(Transport, Packet)> {
        let transports = self.transports.lock().unwrap();
        let (tx, rx) = mpsc::channel(100);

        for transport in transports.values() {
            transport.init_recv(tx.clone());

            log::debug!(
                "SIP {} transport started, listening on {}:{}",
                transport.protocol(),
                transport.local_name().host,
                transport.local_name().port.unwrap()
            );
        }

        rx
    }

    pub async fn recv_packet(
        &self,
        endpoint: &Endpoint,
    ) -> io::Result<()> {
        let mut rx = self.initialize();
        while let Some(msg) = rx.recv().await {
            let (transport, packet) = msg;
            let msg = match packet.payload() {
                CRLF => {
                    transport.send(END, packet.addr()).await?;
                    continue;
                }
                END => {
                    // do nothing
                    continue;
                }
                bytes => match parser::parse_sip_msg(bytes) {
                    Ok(mut sip) => {
                        if let SipMsg::Request(ref mut req) = sip {
                            req.req_headers
                                .as_mut()
                                .unwrap()
                                .via
                                .received = Some(packet.addr().ip());
                        };
                        sip
                    }
                    Err(err) => {
                        log::warn!(
                                "Ignoring {} bytes packet from {} {} : {}\n{}-- end of packet.",
                                packet.payload().len(),
                                transport.protocol(),
                                packet.addr(),
                                err.message,
                                packet.to_string()
                            );
                        continue;
                    }
                },
            };

            let info = IncomingInfo::new(packet, transport);
            let msg = match msg {
                SipMsg::Request(req) => IncomingMessage::Request(
                    IncomingRequest::new(req, info),
                ),
                SipMsg::Response(res) => IncomingMessage::Response(
                    IncomingResponse::new(res, info),
                ),
            };
            endpoint.on_transport_msg(msg).await?;
        }

        Ok(())
    }
}

#[async_trait]
pub trait SipTransport: Sync + Send + 'static {
    async fn send(
        &self,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> io::Result<usize>;

    fn init_recv(&self, sender: TpSender);

    fn protocol(&self) -> TransportProtocol;

    fn addr(&self) -> SocketAddr;

    fn is_same_address_family(&self, addr: &SocketAddr) -> bool {
        let our_addr = self.addr();
        (addr.is_ipv4() && our_addr.is_ipv4())
            || (addr.is_ipv6() && our_addr.is_ipv6())
    }

    fn local_name(&self) -> &HostPort;

    fn reliable(&self) -> bool;

    fn secure(&self) -> bool;

    fn key(&self) -> ConnectionKey {
        ConnectionKey::new(self.addr(), self.protocol())
    }
}

#[derive(Clone)]
pub struct Transport(Arc<dyn SipTransport>);

impl Transport {
    pub fn new(transport: impl SipTransport) -> Self {
        Self(Arc::new(transport))
    }

    pub(crate) async fn send_msg(
        &self,
        addr: &SocketAddr,
        msg: &OutgoingResponse,
    ) -> io::Result<()> {
        let Some(ref buf) = msg.buf else {
            todo!("Return error!");
        };
        let sended = self.send(buf, addr).await?;
        if sended > 0 {
            log::trace!(
                "Response msg {} {} sent successfully {} bytes",
                msg.status_code().into_u32(),
                msg.rphrase(),
                sended
            );
        }
        Ok(())
    }
}

impl std::fmt::Debug for Transport {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        f.debug_struct("Transport")
            .field("addr", &self.addr())
            .field("protocol", &self.protocol())
            .finish()
    }
}

impl PartialOrd for Transport {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transport {
    fn cmp(&self, other: &Self) -> Ordering {
        Arc::strong_count(&self.0).cmp(&Arc::strong_count(&other.0))
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
        self.0.as_ref()
    }
}

impl From<Udp> for Transport {
    fn from(value: Udp) -> Self {
        Transport(Arc::new(value))
    }
}

pub struct Packet {
    pub payload: Arc<[u8]>,
    pub addr: SocketAddr,
    pub time: SystemTime,
}

impl Packet {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn addr(&self) -> &SocketAddr {
        &self.addr
    }
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.payload).to_string()
    }
}

#[derive(Clone)]
pub struct OutgoingInfo {
    pub addr: SocketAddr,
    pub transport: Transport,
}

pub enum IncomingMessage {
    Request(IncomingRequest),
    Response(IncomingResponse),
}

impl IncomingMessage {
    pub fn request(&self) -> Option<&IncomingRequest> {
        if let IncomingMessage::Request(req) = self {
            Some(req)
        } else {
            None
        }
    }
}

pub enum OutgoingMessage {
    Request(OutGoingRequest),
    Response(OutgoingResponse),
}

impl From<OutgoingResponse> for OutgoingMessage {
    fn from(value: OutgoingResponse) -> Self {
        Self::Response(value)
    }
}

impl From<IncomingRequest> for IncomingMessage {
    fn from(value: IncomingRequest) -> Self {
        Self::Request(value)
    }
}

pub struct IncomingInfo {
    packet: Packet,
    pub transport: Transport,
}

impl IncomingInfo {
    pub fn new(packet: Packet, transport: Transport) -> Self {
        Self { packet, transport }
    }

    pub fn packet(&self) -> &Packet {
        &self.packet
    }

    pub fn transport(&self) -> &Transport {
        &self.transport
    }
}

pub struct OutgoingRequest {
    msg: SipRequest,
    info: OutgoingInfo,
}

#[derive(Debug)]
pub struct RequestHeaders {
    pub via: Via,
    pub from: headers::From,
    pub to: To,
    pub callid: CallId,
    pub cseq: CSeq,
}

impl std::fmt::Display for RequestHeaders {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}: {}\r\n", Via::NAME, &self.via)?;
        write!(f, "{}: {}\r\n", headers::From::NAME, self.from)?;
        write!(f, "{}: {}\r\n", To::NAME, self.to)?;
        write!(f, "{}: {}\r\n", CallId::NAME, self.callid)?;
        write!(f, "{}: {}\r\n", CSeq::NAME, self.cseq)?;

        Ok(())
    }
}

pub struct OutgoingResponse {
    pub hdrs: Box<RequestHeaders>,
    pub msg: SipResponse,
    pub info: OutgoingInfo,
    pub buf: Option<Arc<MsgBuffer>>,
}

impl OutgoingResponse {
    pub fn status_code(&self) -> StatusCode {
        self.msg.st_line.code
    }

    pub fn rphrase(&self) -> &str {
        &self.msg.st_line.rphrase
    }

    pub fn is_provisional(&self) -> bool {
        self.msg.st_line.code.is_provisional()
    }

    pub fn into_buffer(&self) -> io::Result<MsgBuffer> {
        let mut buf = MsgBuffer::new();

        write!(buf, "{}", &self.msg.st_line)?;
        write!(buf, "{}", &self.hdrs)?;
        write!(buf, "{}\r\n", &self.msg.headers)?;

        if let Some(body) = &self.msg.body {
            buf.extend_from_slice(body);
        }

        Ok(buf)
    }
}

pub struct OutGoingRequest {
    pub msg: SipRequest,
    pub info: OutgoingInfo,
    pub buf: Option<Arc<MsgBuffer>>,
}

pub struct IncomingRequest {
    pub msg: SipRequest,
    pub info: IncomingInfo,
}

impl IncomingRequest {
    pub fn new(msg: SipRequest, info: IncomingInfo) -> Self {
        Self { msg, info }
    }

    pub fn is_method(&self, method: &SipMethod) -> bool {
        self.msg.method() == method
    }

    pub fn method(&self) -> &SipMethod {
        self.msg.method()
    }

    pub fn addr(&self) -> &SocketAddr {
        &self.info.packet.addr
    }
}

pub struct IncomingResponse {
    pub msg: SipResponse,
    pub info: IncomingInfo,
}

impl IncomingResponse {
    pub fn packet(&self) -> &Packet {
        &self.info.packet
    }
    pub fn transport(&self) -> &Transport {
        &self.info.transport
    }

    pub fn request(&self) -> &SipResponse {
        &self.msg
    }

    pub fn code(&self) -> StatusCode {
        self.msg.st_line.code
    }
}

impl IncomingResponse {
    pub fn new(msg: SipResponse, info: IncomingInfo) -> Self {
        Self { msg, info }
    }
}

impl IncomingRequest {
    pub fn packet(&self) -> &Packet {
        &self.info.packet
    }
    pub fn transport(&self) -> &Transport {
        &self.info.transport
    }

    pub fn request(&self) -> &SipRequest {
        &self.msg
    }
}
