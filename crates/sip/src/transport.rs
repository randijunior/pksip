use std::{
    cmp::Ordering,
    io::{self},
    net::SocketAddr,
    ops::Deref,
    sync::Arc,
    time::SystemTime,
};

pub mod manager;
pub mod udp;

use arrayvec::ArrayVec;
use async_trait::async_trait;
use manager::ConnectionKey;
use std::io::Write;
use tokio::sync::mpsc;
use udp::Udp;

use crate::msg::{SipRequest, SipResponse, TransportProtocol};

pub(crate) const MAX_PACKET_SIZE: usize = 4000;
// pub(crate) type MsgBuffer = ArrayVec<u8, MAX_PACKET_SIZE>;

#[derive(Default)]
pub(crate) struct MsgBuffer(ArrayVec<u8, MAX_PACKET_SIZE>);

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
        self.0
            .try_extend_from_slice(data)
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Buffer full"))
    }

    pub fn write<T>(&mut self, data: T) -> Result<(), io::Error>
    where
        T: std::fmt::Display,
    {
        write!(self.0, "{}", data)
    }
}

#[async_trait]
pub trait SipTransport: Sync + Send + 'static {
    async fn send(&self, pkt: &[u8], dest: SocketAddr) -> io::Result<usize>;

    fn spawn(&self, sender: mpsc::Sender<(Transport, Packet)>);
    fn get_protocol(&self) -> TransportProtocol;
    fn get_addr(&self) -> SocketAddr;
    fn is_same_address_family(&self, addr: &SocketAddr) -> bool;

    fn reliable(&self) -> bool;

    fn secure(&self) -> bool;

    fn get_key(&self) -> ConnectionKey;
}

#[derive(Clone)]
pub struct Transport {
    inner: Arc<dyn SipTransport>,
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
        self.get_key() == other.get_key()
    }
}

impl Deref for Transport {
    type Target = dyn SipTransport;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl From<Udp> for Transport {
    fn from(value: Udp) -> Self {
        Transport {
            inner: Arc::new(value),
        }
    }
}

pub struct Packet {
    payload: Arc<[u8]>,
    addr: SocketAddr,
    time: SystemTime,
}

impl Packet {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

pub struct OutgoingInfo {
    pub addr: SocketAddr,
    pub transport: Transport,
}

pub struct IncomingInfo {
    packet: Packet,
    transport: Transport,
}

pub struct OutGoingRequest<'a> {
    msg: SipRequest<'a>,
    info: OutgoingInfo,
}

pub struct OutGoingResponse<'a> {
    pub msg: SipResponse<'a>,
    pub info: OutgoingInfo,
}

pub struct IncomingRequest<'a> {
    msg: SipRequest<'a>,
    info: IncomingInfo,
}

impl<'a> IncomingRequest<'a> {
    pub fn new(msg: SipRequest<'a>, info: IncomingInfo) -> Self {
        Self { msg, info }
    }
}

pub struct IncomingResponse<'a> {
    msg: SipResponse<'a>,
    info: IncomingInfo,
}

impl<'a> IncomingResponse<'a> {
    pub fn new(msg: SipResponse<'a>, info: IncomingInfo) -> Self {
        Self { msg, info }
    }
}

impl<'a> IncomingRequest<'a> {
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
