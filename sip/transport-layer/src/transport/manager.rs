use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc::{self};

use encoding_layer::msg::TransportProtocol;

use super::{
    MsgBuffer, OutgoingInfo, Packet, SipTransport, Transport, TxResponse,
};

pub const CRLF: &[u8] = b"\r\n";
pub const END: &[u8] = b"\r\n\r\n";

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

pub struct TransportLayer {
    transports: Mutex<HashMap<ConnectionKey, Transport>>,
}

impl Default for TransportLayer {
    fn default() -> Self {
        Self {
            transports: Mutex::new(HashMap::new()),
        }
    }
}

impl TransportLayer {
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

        tps.insert(transport.get_key(), Transport(Arc::clone(&transport.0)));
    }

    pub async fn send_response<'a>(
        &self,
        resp: &mut TxResponse<'a>,
    ) -> io::Result<()> {
        let mut buf = MsgBuffer::new();

        buf.write(&resp.msg.st_line)?;
        buf.write(&resp.req_hdrs)?;
        buf.write(&resp.msg.headers)?;
        buf.write("\r\n")?;
        if let Some(body) = resp.msg.body {
            if let Err(_err) = buf.try_extend_from_slice(body) {
                return Err(io::Error::other(
                    "Packet size exceeds MAX_PACKET_SIZE",
                ));
            }
        }
        let OutgoingInfo { addr, transport } = &resp.info;
        let _sent = transport.send(&buf, *addr).await?;

        resp.buf = Some(buf);

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

    pub fn start(&self) -> mpsc::Receiver<(Transport, Packet)> {
        let transports = self.transports.lock().unwrap();
        let (tx, rx) = mpsc::channel(1024);

        for transport in transports.values() {
            transport.spawn(tx.clone());
        }

        rx
    }
}

#[cfg(test)]
mod tests {}
