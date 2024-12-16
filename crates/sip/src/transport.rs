use std::{
    cmp::Ordering, io, net::SocketAddr, ops::Deref, sync::Arc, time::SystemTime,
};

pub mod manager;
pub mod udp;

use async_trait::async_trait;
use manager::ConnectionKey;
use tokio::sync::mpsc;
use udp::Udp;

use crate::msg::{SipMessage, TransportProtocol};

const MAX_PACKET_SIZE: usize = 4000;

#[async_trait]
pub trait SipTransport: Sync + Send + 'static {
    async fn send(&self, pkt: &[u8], dest: SocketAddr) -> io::Result<usize>;
    async fn recv(&self, pkt: &mut [u8]) -> io::Result<(usize, SocketAddr)>;

    fn spawn(&self, sender: mpsc::Sender<(Transport, Packet)>);
    fn get_protocol(&self) -> TransportProtocol;
    fn get_addr(&self) -> SocketAddr;
    fn is_same_address_family(&self, addr: &SocketAddr) -> bool;

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
    buf: Arc<[u8]>,
    addr: SocketAddr,
    time: SystemTime,
}

impl Packet {
    pub fn buf(&self) -> &[u8] {
        &self.buf
    }
}

pub struct IncomingMessage<'a> {
    packet: Packet,
    msg: SipMessage<'a>,
    transport: Transport,
}

impl<'a> IncomingMessage<'a> {
    pub fn packet(&self) -> &Packet {
        &self.packet
    }
}

// packetize or encode or serialize
pub trait Serializable {
    fn serialize<'a>(&self) -> &'a [u8];
}
