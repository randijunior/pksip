use async_trait::async_trait;
use local_ip_address::local_ip;
use std::{io, net::SocketAddr, sync::Arc, time::SystemTime};
use tokio::net::{ToSocketAddrs, UdpSocket};

use crate::message::{Host, HostPort, TransportProtocol};

use super::{
    Packet, SipTransport, TpSender, Transport, MAX_PACKET_SIZE,
};

#[derive(Debug)]
pub struct Inner {
    pub sock: UdpSocket,
    pub addr: SocketAddr,
    pub local_name: HostPort,
}

#[derive(Debug, Clone)]
pub struct Udp(Arc<Inner>);

#[async_trait]
impl SipTransport for Udp {
    async fn send(
        &self,
        buf: &[u8],
        addr: &SocketAddr,
    ) -> io::Result<usize> {
        self.0.sock.send_to(buf, addr).await
    }

    fn init_recv(&self, sender: TpSender) {
        let udp = Arc::new(self.clone());
        tokio::spawn(Box::pin(Udp::recv_from(udp, sender)));
    }

    fn protocol(&self) -> TransportProtocol {
        TransportProtocol::UDP
    }

    fn reliable(&self) -> bool {
        false
    }

    fn secure(&self) -> bool {
        false
    }

    fn addr(&self) -> SocketAddr {
        self.0.addr
    }

    fn local_name(&self) -> &HostPort {
        &self.0.local_name
    }
}

impl Udp {
    pub async fn bind<A: ToSocketAddrs>(
        addr: A,
    ) -> io::Result<Transport> {
        let sock = UdpSocket::bind(addr).await?;
        let addr = sock.local_addr()?;
        let local_name = HostPort {
            host: Host::IpAddr(local_ip().unwrap_or(addr.ip())),
            port: Some(addr.port()),
        };
        let udp = Arc::new(Inner {
            sock,
            addr,
            local_name,
        });

        Ok(Udp(udp).into())
    }

    async fn recv_from(
        udp: Arc<Self>,
        sender: TpSender,
    ) -> io::Result<()> {
        let mut buf = vec![0u8; MAX_PACKET_SIZE];
        loop {
            let (len, addr) = udp.0.sock.recv_from(&mut buf).await?;
            let buf = &buf[..len];
            let packet = Packet {
                time: SystemTime::now(),
                payload: buf.into(),
                addr,
            };
            let tp = Transport(udp.clone() as Arc<dyn SipTransport>);
            sender.send((tp, packet)).await.unwrap();
        }
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use super::*;
    use crate::{
        message::TransportProtocol,
        transport::{SipTransport, TpSender},
    };

    pub struct MockUdpTransport;

    #[async_trait]
    impl SipTransport for MockUdpTransport {
        async fn send(
            &self,
            buf: &[u8],
            _addr: &SocketAddr,
        ) -> io::Result<usize> {
            Ok(buf.len())
        }

        fn init_recv(&self, _sender: TpSender) {
            // Mock implementation
        }

        fn protocol(&self) -> TransportProtocol {
            TransportProtocol::UDP
        }

        fn addr(&self) -> SocketAddr {
            "127.0.0.1:5060".parse().unwrap()
        }

        fn reliable(&self) -> bool {
            false
        }

        fn secure(&self) -> bool {
            false
        }
        fn local_name(&self) -> &HostPort {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    #[tokio::test]
    async fn test_udp() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let udp_transport = Udp::bind(addr).await.unwrap();
        let ipv4_addr = "192.168.0.1:8080".parse().unwrap();

        assert!(udp_transport.is_same_address_family(&ipv4_addr));
        assert_eq!(udp_transport.protocol(), TransportProtocol::UDP);

        let client = UdpSocket::bind(addr).await.unwrap();
        let client_addr = client.local_addr().unwrap();
        let buf = b"hello world";

        let (tx, mut rx) = mpsc::channel(100);

        udp_transport.init_recv(tx);

        client.send_to(buf, udp_transport.addr()).await.unwrap();
        let (transport, packet) = rx.recv().await.unwrap();

        assert!(transport == udp_transport);
        assert_eq!(packet.payload(), buf);
        assert_eq!(packet.addr, client_addr);
    }
}
