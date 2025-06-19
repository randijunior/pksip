// Temporarily allow unused imports and dead code warnings.
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use super::{
    decoder::StreamingDecoder, Direction, Factory, Packet, SipTransport, Transport, TransportKey, TransportStartup,
    TransportTx,
};
use crate::{
    error::{Error, Result},
    message::TransportKind,
    transport::{TransportEvent, TransportPacket},
    Endpoint,
};
use local_ip_address::local_ip;
use std::{borrow::Cow, net::SocketAddr, sync::Arc, time::SystemTime};
use tokio::{
    io::{split, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::Mutex,
};
use tokio_stream::StreamExt;
use tokio_util::codec::FramedRead;

type TcpRead = FramedRead<ReadHalf<TcpStream>, StreamingDecoder>;
type TcpWrite = Arc<Mutex<WriteHalf<TcpStream>>>;

#[derive(Clone)]
/// TCP transport implementation.
pub struct TcpTransport {
    /// The transport addr.
    addr: SocketAddr,
    /// The transport remote addr.
    remote_addr: SocketAddr,

    /// The tcp writer.
    write: TcpWrite,

    /// Transport direction.
    dir: Direction,
}

#[async_trait::async_trait]
impl SipTransport for TcpTransport {
    async fn send(&self, buf: &[u8], _: &SocketAddr) -> Result<usize> {
        let mut writer = self.write.lock().await;

        writer.write_all(buf).await?;
        writer.flush().await?;

        Ok(buf.len())
    }

    fn tp_kind(&self) -> TransportKind {
        TransportKind::Tcp
    }

    fn addr(&self) -> SocketAddr {
        self.addr
    }

    fn local_name(&self) -> Cow<'_, str> {
        Cow::Owned(self.addr.to_string())
    }

    fn reliable(&self) -> bool {
        true
    }

    fn secure(&self) -> bool {
        false
    }
}

/// A TCP server for accept incoming connections.
pub struct TcpServer {
    // Main socket for accept tcp connections.
    sock: TcpListener,
    // Where this server is bind to.
    addr: SocketAddr,
    // The server local name addres.
    local_name: String,
}

struct TcpStreamRead {
    reader: TcpRead,
    addr: SocketAddr,
    transport: Transport,
    sender: TransportTx,
}

impl TcpServer {
    /// Creates a new TCP server.
    pub async fn create<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let sock = TcpListener::bind(addr).await?;
        let addr = sock.local_addr()?;
        let local_name = crate::get_local_name(&addr);

        Ok(Self { sock, local_name, addr })
    }

    /// Serves incoming TCP connections by accepting and handling them.
    pub(crate) async fn handle_incoming(self, sender: TransportTx) -> Result<()> {
        loop {
            let (stream, addr) = match self.sock.accept().await {
                Ok(ok) => ok,
                Err(err) => {
                    log::error!("Failed to accept connection: {:#}", err);
                    continue;
                }
            };
            #[cfg(test)]
            println!("Got incoming TCP connection from {}", addr);

            log::debug!("Got incoming TCP connection from {}", addr);
            // Spawn a new task to handle the connection.
            tokio::spawn(Self::on_accept((stream, addr), sender.clone()));
        }
    }

    // Handle incoming connection.
    async fn on_accept((stream, addr): (TcpStream, SocketAddr), sender: TransportTx) -> Result<()> {
        let local_addr = stream.local_addr()?;
        let (read, write) = split(stream);
        let decoder = StreamingDecoder;

        let reader = FramedRead::new(read, decoder);
        let write = Arc::new(Mutex::new(write));

        // Create TCP transport for the new socket.
        let transport = Transport::new(TcpTransport {
            addr: local_addr,
            remote_addr: addr,
            write,
            dir: Direction::Incoming,
        });

        // Register the new transport.
        sender.send(TransportEvent::TransportCreated(transport.clone())).await?;

        let reader = TcpStreamRead {
            reader,
            addr,
            transport,
            sender,
        };

        tokio::spawn(async move {
            if let Err(err) = Self::stream_read(reader).await {
                log::warn!("An error occured; error = {:#}", err);
            }
        });

        Ok(())
    }

    async fn stream_read(reader: TcpStreamRead) -> Result<()> {
        let TcpStreamRead {
            mut reader,
            addr,
            transport,
            sender,
        } = reader;
        let key = transport.key();

        loop {
            match reader.next().await {
                Some(Ok(payload)) => {
                    let time = SystemTime::now();
                    let packet = Packet { payload, addr, time };
                    let transport = transport.clone();
                    let msg = TransportPacket  {
                        transport,
                        packet
                    };

                    // Send.
                    sender.send(TransportEvent::PacketReceived(msg)).await?;
                }
                Some(Err(err)) => {
                    return Err(Error::Io(err));
                }
                None => {
                    sender.send(TransportEvent::TransportClosed(key)).await?;
                }
            };
        }
    }
}

#[derive(Clone, Copy, Default)]
/// Factory for create tcp transports.
pub struct TcpFactory;

#[async_trait::async_trait]
impl Factory for TcpFactory {
    async fn create(&self, addr: SocketAddr) -> Result<Transport> {
        // TODO: Keep-Alive timer.
        let stream = TcpStream::connect(addr).await?;
        let addr = stream.local_addr()?;
        let remote_addr = stream.peer_addr()?;

        let (read, write) = split(stream);

        let write = Arc::new(Mutex::new(write));

        Ok(Transport::new(TcpTransport {
            addr,
            remote_addr,
            write,
            dir: Direction::Outgoing,
        }))
    }

    fn transport_kind(&self) -> TransportKind {
        TransportKind::Tcp
    }
}

pub(crate) struct TcpStartup {
    addr: SocketAddr,
}

impl TcpStartup {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}

#[async_trait::async_trait]
impl TransportStartup for TcpStartup {
    async fn start(&self, sender: TransportTx) -> Result<()> {
        let tcp_server = TcpServer::create(self.addr).await?;

        log::debug!(
            "SIP {} transport ready for incoming connections at {}",
            TransportKind::Tcp,
            tcp_server.local_name
        );

        let factory = Box::new(TcpFactory);

        sender.send(TransportEvent::FactoryCreated(factory)).await?;

        tokio::spawn(tcp_server.handle_incoming(sender));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpSocket;

    use crate::transport::TransportPacket;

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
    async fn smoke() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let endpoint = crate::endpoint::Builder::new().build().await;
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);

        let server = TcpServer::create(addr).await.unwrap();
        let socket = TcpSocket::new_v4().unwrap();
        let server_addr = server.addr;

        tokio::spawn(server.handle_incoming(tx));

        let mut client = socket.connect(server_addr).await.unwrap();

        assert!(matches!(rx.recv().await.unwrap(), TransportEvent::TransportCreated(_)));

        client.write_all(MSG_TEST).await.unwrap();
        client.flush().await.unwrap();

        let TransportEvent::PacketReceived(TransportPacket { packet, .. }) = rx.recv().await.unwrap() else {
            unreachable!();
        };

        assert_eq!(packet.payload.0.as_ref(), MSG_TEST);
    }
}
