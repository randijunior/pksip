use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use std::sync::{Arc, Mutex};

use crate::message::{Request, SipMessage};
use crate::parser::Parser;
use crate::transport::{SipTransport, TransportType};

/// A mock transport, for testing purposes
#[derive(Clone)]
pub struct MockTransport {
    sent: Arc<Mutex<Vec<(Vec<u8>, SocketAddr)>>>,
    addr: SocketAddr,
    tp_type: TransportType,
    fail_at: Option<usize>,
}

impl MockTransport {
    pub fn with_transport_type(tp_type: TransportType) -> Self {
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        let port = tp_type.default_port();
        let mock = Self {
            sent: Default::default(),
            addr: SocketAddr::new(ip, port),
            tp_type,
            fail_at: None,
        };

        mock
    }

    pub fn new_udp() -> Self {
        Self::with_transport_type(TransportType::Udp)
    }

    pub fn new_tcp() -> Self {
        Self::with_transport_type(TransportType::Tcp)
    }

    pub fn new_tls() -> Self {
        Self::with_transport_type(TransportType::Tls)
    }

    pub fn sent_count(&self) -> usize {
        self.sent.lock().unwrap().len()
    }

    pub fn get_last_request(&self) -> Option<Request> {
        self.last_sip_msg().map(|msg| {
            if let SipMessage::Request(req) = msg {
                Some(req)
            } else {
                None
            }
        })?
    }

    pub fn last_buffer(&self) -> Option<Vec<u8>> {
        let guard = self.sent.lock().unwrap();
        guard.last().map(|(buff, _)| buff).cloned()
    }

    pub fn last_sip_msg(&self) -> Option<SipMessage> {
        self.last_buffer().map(|b| Parser::parse(&b).unwrap())
    }

    fn push_msg(&self, (buf_vec, address): (Vec<u8>, SocketAddr)) -> usize {
        let mut guard = self.sent.lock().unwrap();
        guard.push((buf_vec, address));
        guard.len()
    }
}

#[async_trait::async_trait]
impl SipTransport for MockTransport {
    async fn send_msg(&self, buf: &[u8], address: &SocketAddr) -> crate::Result<usize> {
        let current_count = self.push_msg((buf.to_vec(), *address));

        if let Some(fail_at) = self.fail_at
            && fail_at == current_count
        {
            return Err(crate::Error::TransportError("Simulated failure".into()));
        }

        Ok(buf.len())
    }

    fn remote_addr(&self) -> Option<SocketAddr> {
        None
    }

    fn protocol(&self) -> TransportType {
        self.tp_type
    }

    fn local_addr(&self) -> SocketAddr {
        self.addr
    }
}
