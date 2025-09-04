//! SIP UDP Transport.
//! This module provides the implementation of the SIP
//! transport layer over UDP.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use tokio::net::ToSocketAddrs;
use tokio::net::UdpSocket;

use super::Packet;
use super::Payload;
use super::Transport;
use super::TransportMessage;
use super::TransportRef;
use super::TransportStartup;
use super::TransportTx;
use super::TransportType;
use crate::error::Result;

#[derive(Debug)]
struct Inner {
    pub sock: UdpSocket,
    pub addr: SocketAddr,
    pub local_name: String,
}

#[derive(Debug, Clone)]
/// UDP transport implementation.
pub struct UdpTransport(Arc<Inner>);

impl UdpTransport {
    /// Binds a UDP transport to the specified address.
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let sock = UdpSocket::bind(addr).await?;

        let addr = sock.local_addr()?;
        let local_name = crate::get_local_name(&addr);

        Ok(Self(Arc::new(Inner {
            sock,
            addr,
            local_name,
        })))
    }

    async fn recv_from(udp: Arc<Self>, sender: TransportTx) -> Result<()> {
        let udp_tp = udp.clone();
        // Buffer to recv packet.
        let mut buf = vec![0u8; 4000];

        loop {
            // Read data into vec.
            let (len, addr) = udp.0.sock.recv_from(&mut buf).await?;

            // Copy buf.
            let datagram_msg = bytes::Bytes::copy_from_slice(&buf[..len]);

            // Create Payload.
            let payload = Payload(datagram_msg);
            let time = SystemTime::now();

            // Create Packet.
            let packet = Packet {
                payload,
                addr,
                time,
            };
            let transport = udp_tp.clone();

            // Send.
            sender
                .send(TransportMessage::Packet { transport, packet })
                .await?;
        }
    }
}

#[async_trait::async_trait]
impl Transport for UdpTransport {
    async fn send(&self, buf: &[u8], addr: &SocketAddr) -> Result<usize> {
        Ok(self.0.sock.send_to(buf, addr).await?)
    }

    fn protocol(&self) -> TransportType {
        TransportType::Udp
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

    fn local_name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(&self.0.local_name)
    }
}

pub(crate) struct UdpStartup {
    addr: SocketAddr,
}

impl UdpStartup {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

#[async_trait::async_trait]
impl TransportStartup for UdpStartup {
    async fn start(&self, sender: TransportTx) -> Result<()> {
        let udp = UdpTransport::bind(self.addr).await?;

        log::debug!(
            "SIP {} transport started, listening on {}",
            TransportType::Udp,
            udp.local_name()
        );

        let arc_udp = Arc::new(udp.clone());
        let transport = Arc::new(udp) as TransportRef;

        sender.send(TransportMessage::Created(transport)).await?;

        tokio::spawn(Box::pin(UdpTransport::recv_from(arc_udp, sender)));

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod mock {
    use super::*;

    pub struct MockUdpTransport;

    #[async_trait::async_trait]
    impl Transport for MockUdpTransport {
        async fn send(&self, buf: &[u8], _addr: &SocketAddr) -> Result<usize> {
            Ok(buf.len())
        }

        fn protocol(&self) -> TransportType {
            TransportType::Udp
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

        fn local_name(&self) -> std::borrow::Cow<'_, str> {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;

    const MSG_TEST: &[u8] = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n\
        Via: SIP/2.0/UDP bobspc.biloxi.com:5060;branch=z9hG4bKnashds7\r\n\
        Max-Forwards: 70\r\n\
        To: Bob <sip:bob@biloxi.com>\r\n\
        From: Bob <sip:bob@biloxi.com>;tag=456248\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Contact: <sip:bob@192.0.2.4>\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n";

    #[tokio::test]
    async fn test_recv_msg() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let (tx, mut rx) = mpsc::channel(1);

        let udp = UdpTransport::bind(addr).await.unwrap();
        let client = UdpSocket::bind(addr).await.unwrap();

        tokio::spawn(UdpTransport::recv_from(Arc::new(udp.clone()), tx));

        client.send_to(MSG_TEST, udp.addr()).await.unwrap();

        let TransportMessage::Packet {
            transport: _,
            packet,
        } = rx.recv().await.unwrap()
        else {
            unreachable!();
        };

        assert_eq!(packet.payload.buf(), MSG_TEST);

        let client_addr = client.local_addr().unwrap();
        assert_eq!(packet.addr, client_addr);
    }

    #[tokio::test]
    async fn test_send_msg() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

        let udp = UdpTransport::bind(addr).await.unwrap();
        let client = UdpSocket::bind(addr).await.unwrap();

        let client_addr = client.local_addr().unwrap();

        udp.send(MSG_TEST, &client_addr).await.unwrap();

        let mut buf = [0; MSG_TEST.len()];
        let len = client.recv(&mut buf).await.unwrap();

        assert!(len == MSG_TEST.len());
        assert_eq!(&buf[..len], MSG_TEST);
    }
}
