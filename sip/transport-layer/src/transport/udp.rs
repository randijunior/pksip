use async_trait::async_trait;
use std::{io, net::SocketAddr, sync::Arc, time::SystemTime};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::mpsc,
};

use encoding_layer::message::TransportProtocol;

use super::{
    manager::ConnectionKey, Packet, SipTransport, Transport, MAX_PACKET_SIZE,
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

    fn spawn(&self, sender: mpsc::Sender<(Transport, Packet)>) {
        tokio::spawn(Udp::receive_packet(self.clone(), sender));
    }

    fn get_protocol(&self) -> TransportProtocol {
        TransportProtocol::UDP
    }

    fn is_same_address_family(&self, addr: &SocketAddr) -> bool {
        (addr.is_ipv4() && self.0.addr.is_ipv4())
            || (addr.is_ipv6() && self.0.addr.is_ipv6())
    }

    fn reliable(&self) -> bool {
        false
    }
    fn secure(&self) -> bool {
        false
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

    pub async fn default() -> Transport {
        Self::bind("0.0.0.0:5060").await.unwrap()
    }

    async fn receive_packet(
        self,
        sender: mpsc::Sender<(Transport, Packet)>,
    ) -> io::Result<()> {
        let mut buf = [0u8; MAX_PACKET_SIZE];
        loop {
            let (len, addr) = self.0.sock.recv_from(&mut buf).await?;
            let buf = &buf[..len];
            let packet = Packet {
                time: SystemTime::now(),
                payload: buf.into(),
                addr,
            };
            sender.send((self.clone().into(), packet)).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let udp_transport = Udp::bind(addr).await.unwrap();
        let ipv4_addr = "192.168.0.1:8080".parse().unwrap();

        assert!(udp_transport.is_same_address_family(&ipv4_addr));
        assert_eq!(udp_transport.get_protocol(), TransportProtocol::UDP);

        let client = UdpSocket::bind(addr).await.unwrap();
        let client_addr = client.local_addr().unwrap();
        let buf = b"hello world";

        let (tx, mut rx) = mpsc::channel(100);

        udp_transport.spawn(tx);

        client.send_to(buf, udp_transport.get_addr()).await.unwrap();
        let (transport, packet) = rx.recv().await.unwrap();

        assert!(transport == udp_transport);
        assert_eq!(packet.payload(), buf);
        assert_eq!(packet.addr, client_addr);
    }
}
