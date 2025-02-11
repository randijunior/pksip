use std::{
    cmp::Ordering,
    collections::HashMap,
    io::{self},
    net::SocketAddr,
    ops::Deref,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub mod udp;

use arrayvec::ArrayVec;
use async_trait::async_trait;

use std::io::Write;
use tokio::sync::mpsc;
use udp::Udp;

use crate::{
    filter_map_header, find_map_header,
    headers::{self, CSeq, CallId, Headers, SipHeader, To, Via},
    message::{
        SipMethod, SipRequest, SipResponse, StatusCode,
        TransportProtocol,
    },
};

pub(crate) const CRLF: &[u8] = b"\r\n";
pub(crate) const END: &[u8] = b"\r\n\r\n";
pub(crate) const MAX_PACKET_SIZE: usize = 4000;

pub(crate) type TransportSender = mpsc::Sender<(Transport, Packet)>;

#[derive(Default, Debug)]
pub struct MsgBuffer(ArrayVec<u8, MAX_PACKET_SIZE>);

impl Deref for MsgBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MsgBuffer {
    pub fn new() -> Self {
        Self(ArrayVec::new())
    }

    pub fn try_extend_from_slice(
        &mut self,
        data: &[u8],
    ) -> Result<(), io::Error> {
        self.0.try_extend_from_slice(data).map_err(|_| {
            io::Error::new(io::ErrorKind::Other, "Buffer full")
        })
    }

    pub fn write<T>(&mut self, data: T) -> Result<(), io::Error>
    where
        T: std::fmt::Display,
    {
        write!(self.0, "{}", data)
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

    pub async fn send_response(
        &self,
        resp: &OutgoingResponse,
    ) -> io::Result<()> {
        let OutgoingInfo { addr, transport } = &resp.info;
        let buf = resp.into_buffer()?;
        let _sent = transport.send(&buf, *addr).await?;

        Ok(())
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
        let (tx, rx) = mpsc::channel(1024);

        for transport in transports.values() {
            transport.spawn(tx.clone());
        }

        rx
    }
}

#[async_trait]
pub trait SipTransport: Sync + Send + 'static {
    async fn send(
        &self,
        pkt: &[u8],
        dest: SocketAddr,
    ) -> io::Result<usize>;

    fn spawn(&self, sender: TransportSender);

    fn protocol(&self) -> TransportProtocol;

    fn addr(&self) -> SocketAddr;

    fn is_same_address_family(&self, addr: &SocketAddr) -> bool {
        let our_addr = self.addr();
        (addr.is_ipv4() && our_addr.is_ipv4())
            || (addr.is_ipv6() && our_addr.is_ipv6())
    }

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
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[derive(Clone)]
pub struct OutgoingInfo {
    pub addr: SocketAddr,
    pub transport: Transport,
}

pub struct IncomingInfo {
    packet: Packet,
    transport: Transport,
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

pub struct RequestHeaders {
    pub via: Vec<Via>,
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
        for via in &self.via {
            write!(f, "{}: {}\r\n", Via::NAME, via)?;
        }
        write!(f, "{}: {}\r\n", headers::From::NAME, self.from)?;
        write!(f, "{}: {}\r\n", To::NAME, self.to)?;
        write!(f, "{}: {}\r\n", CallId::NAME, self.callid)?;
        write!(f, "{}: {}\r\n", CSeq::NAME, self.cseq)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct MissingHeaderError;

impl TryFrom<&Headers> for RequestHeaders {
    type Error = MissingHeaderError;

    fn try_from(hdrs: &Headers) -> Result<Self, Self::Error> {
        let via = filter_map_header!(hdrs, Via);
        let from = find_map_header!(hdrs, From);
        let to = find_map_header!(hdrs, To);
        let callid = find_map_header!(hdrs, CallId);
        let cseq = find_map_header!(hdrs, CSeq);

        let via: Vec<Via> = via.cloned().collect();

        if via.is_empty() {
            return Err(MissingHeaderError);
        }

        let from = from.ok_or(MissingHeaderError)?;
        let to = to.ok_or(MissingHeaderError)?;
        let callid = callid.ok_or(MissingHeaderError)?;
        let cseq = cseq.ok_or(MissingHeaderError)?;

        Ok(Self {
            via,
            from: from.clone(),
            to: to.clone(),
            callid: callid.clone(),
            cseq: cseq.clone(),
        })
    }
}

pub struct OutgoingResponse {
    pub req_hdrs: RequestHeaders,
    pub msg: SipResponse,
    pub info: OutgoingInfo,
    pub buf: Option<Arc<MsgBuffer>>,
}

impl<'a> OutgoingResponse {
    pub fn status_code(&self) -> StatusCode {
        self.msg.st_line.code
    }
    pub fn code_num(&self) -> u32 {
        self.msg.st_line.code as u32
    }
    pub fn is_provisional(&self) -> bool {
        self.msg.st_line.code.is_provisional()
    }

    pub fn into_buffer(&self) -> io::Result<MsgBuffer> {
        let mut buf = MsgBuffer::new();

        buf.write(&self.msg.st_line)?;
        buf.write(&self.req_hdrs)?;
        buf.write(&self.msg.headers)?;
        buf.write("\r\n")?;
        if let Some(body) = &self.msg.body {
            if let Err(_err) = buf.try_extend_from_slice(&*body) {
                return Err(io::Error::other(
                    "Packet size exceeds MAX_PACKET_SIZE",
                ));
            }
        }

        Ok(buf)
    }
}

pub struct OutGoingRequest {
    pub msg: SipRequest,
    pub info: OutgoingInfo,
    pub buf: Option<Arc<MsgBuffer>>,
}

pub struct ReceivedRequest {
    pub msg: SipRequest,
    pub req_hdrs: Option<RequestHeaders>,
    pub info: IncomingInfo,
}

impl ReceivedRequest {
    pub fn new(
        msg: SipRequest,
        info: IncomingInfo,
        req_hdrs: Option<RequestHeaders>,
    ) -> Self {
        Self {
            msg,
            info,
            req_hdrs,
        }
    }

    pub fn is_method(&self, method: &SipMethod) -> bool {
        self.msg.method() == *method
    }
}

pub struct IncomingResponse {
    msg: SipResponse,
    info: IncomingInfo,
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

impl ReceivedRequest {
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
