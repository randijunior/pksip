use async_trait::async_trait;
use std::{io, net::SocketAddr, sync::Arc, time::SystemTime};
use tokio::{net::{ToSocketAddrs, UdpSocket}, sync::mpsc};

use crate::msg::TransportProtocol;

use super::{
    manager::{ConnectionKey, Transport},
    Packet, SipTransport, MAX_PACKET_SIZE,
};

#[derive(Debug)]
pub struct Inner {
    pub sock: UdpSocket,
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct Udp(Arc<Inner>);

#[async_trait]
impl SipTransport for Udp {
    async fn send(&self, buf: &[u8], dest: SocketAddr) -> io::Result<usize> {
        self.0.sock.send_to(buf, dest).await
    }

    async fn recv(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.0.sock.recv_from(buf).await
    }

    fn spawn(&self, sender: tokio::sync::mpsc::Sender<(Transport, Packet)>) {
        tokio::spawn(Udp::receive_packet(self.clone(), sender));
    }

    fn get_protocol(&self) -> TransportProtocol {
        TransportProtocol::UDP
    }

    fn is_same_address_family(&self, addr: &SocketAddr) -> bool {
        (addr.is_ipv4() && self.0.addr.is_ipv4())
            || (addr.is_ipv6() && self.0.addr.is_ipv6())
    }

    fn get_key(&self) -> ConnectionKey {
        ConnectionKey::from_tp(self)
    }

    fn get_addr(&self) -> SocketAddr {
        self.0.addr
    }
}

impl Udp {
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Transport> {
        let sock = UdpSocket::bind(addr).await?;
        let addr = sock.local_addr()?;
        let udp = Arc::new(Inner { sock, addr });

        Ok(Udp(udp).into())
    }

    async fn receive_packet(
        self,
        sender: mpsc::Sender<(Transport, Packet)>,
    ) -> io::Result<()> {
        let mut buf = [0u8; MAX_PACKET_SIZE];
        loop {
            let (len, addr) = self.recv(&mut buf).await?;
            let buf = &buf[..len];
            let packet = Packet {
                time: SystemTime::now(),
                buf: buf.into(),
                addr,
            };
            sender.send((self.clone().into(), packet)).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR: &str = "127.0.0.1:0";

    async fn test_udp(addr: SocketAddr) {
        let udp = Udp::bind(addr).await.unwrap();
    }

    #[tokio::test]
    async fn test_create_udp() {
        test_udp(ADDR.parse().unwrap()).await
    }
}
