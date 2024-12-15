use std::{io, net::SocketAddr, sync::Arc, time::SystemTime};

pub mod manager;
pub mod udp;
pub mod resolver;

use async_trait::async_trait;
use manager::{ConnectionKey, Transport};
use tokio::sync::mpsc;

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
    transport: Transport
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
