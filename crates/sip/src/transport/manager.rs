use std::{
    cmp::Ordering,
    collections::HashMap,
    io,
    net::SocketAddr,
    ops::Deref,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::{self};

use crate::{
    endpoint::Endpoint, msg::TransportProtocol, parser::parse_sip_msg,
};

use super::{udp::Udp, IncomingMessage, Packet, SipTransport};

const KEEP_ALIVE_REQUEST: &[u8] = b"\r\n\r\n";
const KEEP_ALIVE_RESPONSE: &[u8] = b"\r\n";

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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ConnectionKey {
    addr: SocketAddr,
    protocol: TransportProtocol,
}


impl ConnectionKey {
    pub fn new(addr: SocketAddr, protocol: TransportProtocol) -> Self {
        ConnectionKey { addr, protocol }
    }
    pub fn from_tp(tp: &dyn SipTransport) -> Self {
        ConnectionKey {
            addr: tp.get_addr(),
            protocol: tp.get_protocol(),
        }
    }
}

pub struct TransportManager {
    transports: Mutex<HashMap<ConnectionKey, Transport>>,
}

impl Default for TransportManager {
    fn default() -> Self {
        Self {
            transports: Mutex::new(HashMap::new()),
        }
    }
}

impl TransportManager {
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
            transport.get_key(),
            Transport {
                inner: Arc::clone(&transport.inner),
            },
        );
    }

    pub fn find_tp(
        &self,
        dst: SocketAddr,
        protocol: TransportProtocol,
    ) -> Option<Transport> {
        println!("Finding suitable transport={} for={}", protocol, dst);
        let transports = self.transports.lock().unwrap();

        // find by remote addr
        let key = ConnectionKey::new(dst, protocol);
        if let Some(tp) = transports.get(&key) {
            return Some(tp.clone());
        };

        // Find by transport protocol and address family
        transports
            .values()
            .filter(|transport| {
                transport.get_protocol() == protocol
                    && transport.is_same_address_family(&dst)
            })
            .min_by(|a, b| a.cmp(b))
            .cloned()
    }

    fn start(&self) -> mpsc::Receiver<(Transport, Packet)> {
        let transports = self.transports.lock().unwrap();
        let (tx, rx) = mpsc::channel(1024);

        for transport in transports.values() {
            transport.spawn(tx.clone());
        }

        rx
    }

    pub async fn receive_packet(&self, endpt: &Endpoint) -> io::Result<()> {
        let mut rx = self.start();
        while let Some(tp_msg) = rx.recv().await {
            let (tp, packet) = tp_msg;
            let endpt = endpt.clone();
            tokio::spawn(async move {
                // Process each packet concurrently.
                Self::process_packet(tp, packet, endpt).await
            });
        }

        Ok(())
    }

    async fn process_packet(
        transport: Transport,
        pkt: Packet,
        endpt: Endpoint,
    ) -> io::Result<()> {
        let msg = match pkt.buf.as_ref() {
            KEEP_ALIVE_REQUEST => {
                transport.send(KEEP_ALIVE_RESPONSE, pkt.addr).await?;
                return Ok(());
            }
            KEEP_ALIVE_RESPONSE => {
                return Ok(());
            }
            bytes => match parse_sip_msg(bytes) {
                //Required Headers:  cid, from, to, via, cseq
                Ok(sip) => sip,
                Err(_) => todo!(),
            },
        };
        let msg = IncomingMessage {
            packet: Packet {
                buf: Arc::clone(&pkt.buf),
                ..pkt
            },
            msg,
            transport,
        };
        endpt.endpt_recv_msg(msg).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
