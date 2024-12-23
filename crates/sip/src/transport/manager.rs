use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::{self};

use crate::{
    msg::{SipMessage, TransportProtocol},
    parser::parse_sip_msg,
    server::SipServer,
};

use super::{
    IncomingInfo, IncomingRequest, IncomingResponse, OutGoingResponse,
    OutgoingInfo, Packet, SipTransport, Transport,
};

const CRLF: &[u8] = b"\r\n";
const END: &[u8] = b"\r\n\r\n";

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

    pub async fn send_response<'a>(
        &self,
        resp: OutGoingResponse<'a>,
    ) -> io::Result<()> {
        let buf = resp.msg.encode()?;
        let OutgoingInfo { addr, transport } = resp.info;
        let _sent = transport.send(&buf, addr).await?;

        Ok(())
    }

    pub fn find(
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
        // TODO: create transport if tcp or tls
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

    pub async fn recv(&self, sip_server: &SipServer) -> io::Result<()> {
        let mut rx = self.start();
        while let Some(tp_msg) = rx.recv().await {
            let (tp, packet) = tp_msg;
            let sip_server = sip_server.clone();
            tokio::spawn(async move {
                // Process each packet concurrently.
                if let Err(err) =
                    Self::process_packet(tp, packet, sip_server).await
                {
                    println!("Error on process packet: {:#?}", err);
                }
            });
        }

        Ok(())
    }

    async fn process_packet(
        transport: Transport,
        pkt: Packet,
        sip_server: SipServer,
    ) -> io::Result<()> {
        let msg = match pkt.payload.as_ref() {
            CRLF => {
                transport.send(END, pkt.addr).await?;
                return Ok(());
            }
            END => {
                return Ok(());
            }
            bytes => match parse_sip_msg(bytes) {
                Ok(sip) => sip,
                Err(err) => return Err(io::Error::other(err.message)),
            },
        };
        let info = IncomingInfo {
            packet: Packet {
                payload: Arc::clone(&pkt.payload),
                ..pkt
            },
            transport,
        };
        match msg {
            SipMessage::Request(msg) => {
                let req = IncomingRequest::new(msg, info);
                sip_server.sip_server_recv_req(req).await;
            }
            SipMessage::Response(msg) => {
                let msg = IncomingResponse::new(msg, info);
                sip_server.sip_server_recv_res(msg).await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
