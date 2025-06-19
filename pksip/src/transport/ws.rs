// Temporarily allow unused imports and dead code warnings.
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::SystemTime;

use super::{Direction, SipTransport, Transport, TransportTx};
use crate::message::TransportKind;
use crate::transport::{ws, Packet, Payload, TransportEvent, TransportPacket};
use crate::{error::Result, Endpoint};
use futures_util::stream::SplitSink;
use futures_util::{future, StreamExt, TryStreamExt};
use futures_util::{pin_mut, SinkExt};
use hyper::header::SEC_WEBSOCKET_PROTOCOL;
use hyper::{
    body::Incoming,
    header::{SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION},
    server::conn::http1,
    service::service_fn,
    upgrade::Upgraded,
    Request, Response,
};
use hyper_util::rt::TokioIo;
use tokio::net::{TcpListener, ToSocketAddrs};

use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Role;
use tokio_tungstenite::{tungstenite::handshake::derive_accept_key, WebSocketStream};

use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result as TungsteniteResult},
};

type Body = http_body_util::Full<hyper::body::Bytes>;
type WsWrite = Arc<Mutex<SplitSink<WebSocketStream<TokioIo<Upgraded>>, Message>>>;

/// WebSocket transport implementation.
pub struct WebSocketTransport {
    addr: SocketAddr,
    remote_addr: SocketAddr,
    dir: Direction,
    write: WsWrite,
}

#[async_trait::async_trait]
impl SipTransport for WebSocketTransport {
    async fn send(&self, buf: &[u8], _: &SocketAddr) -> Result<usize> {
        // Convert the buffer into a WebSocket message
        let message = Message::Binary(buf.to_vec().into());

        let mut writer = self.write.lock().await;

        // Send the message through the WebSocket
        writer
            .send(message)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(buf.len())
    }

    fn tp_kind(&self) -> TransportKind {
        TransportKind::Ws
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

/// A WebSocket server for accept incoming connections.
pub struct WebSocketServer {
    sock: TcpListener,
    addr: SocketAddr,
    local_name: String,
}

impl WebSocketServer {
    /// Creates a new TCP server.
    pub async fn bind<A>(addr: A) -> Result<Self>
    where
        A: ToSocketAddrs,
    {
        let sock = TcpListener::bind(addr).await?;
        let addr = sock.local_addr()?;
        let local_name = crate::get_local_name(&addr);

        Ok(Self { sock, addr, local_name })
    }

    async fn on_ws_connection(
        ws_stream: WebSocketStream<TokioIo<Upgraded>>,
        sender: TransportTx,
        addr: SocketAddr,
    ) -> TungsteniteResult<()> {
        println!("WebSocket connection established: {}", addr);

        let (ws_sender, ws_receiver) = ws_stream.split();

        let write = Arc::new(Mutex::new(ws_sender));
        // Create WS transport for the new socket.
        let transport = Transport::new(WebSocketTransport {
            dir: Direction::Incoming,
            addr,
            remote_addr: addr,
            write,
        });

        let _ = sender.send(TransportEvent::TransportCreated(transport.clone())).await;

        let mut filtered_msgs = ws_receiver.try_filter(|msg| {
            // Filter out unwanted messages.
            future::ready(msg.is_text() || msg.is_binary() || msg.is_close())
        });

        while let Some(msg) = filtered_msgs.next().await {
            let payload = match msg? {
                Message::Text(text) => Payload::new(text.into()),
                Message::Binary(bin) => Payload::new(bin),
                Message::Close(_) => {
                    log::debug!("WebSocket connection closed");
                    break;
                }
                _ => {
                    continue;
                }
            };

            let time = SystemTime::now();
            let packet = Packet { payload, addr, time };
            let msg = TransportPacket  {
                transport: transport.clone(),
                packet
            };

            // Send.
            if sender.send(TransportEvent::PacketReceived(msg)).await.is_err() {
                break;
            }
        }

        Ok(())
    }

    /// Handles incoming WebSocket requests.
    async fn on_received_request(
        mut req: Request<Incoming>,
        tx: TransportTx,
        addr: SocketAddr,
    ) -> Result<Response<Body>> {
        // Check if the request is a valid WebSocket handshake
        let headers = req.headers();

        if headers.get(SEC_WEBSOCKET_VERSION).map(|v| v.as_bytes()) != Some(b"13") {
            return Err(crate::error::Error::InvalidWebSocketVersion);
        }

        if headers.get(SEC_WEBSOCKET_PROTOCOL).map(|v| v.as_bytes()) != Some(b"sip") {
            return Err(crate::error::Error::InvalidSecWebSocketProtocol);
        }

        let key = match headers.get(SEC_WEBSOCKET_KEY) {
            Some(key) => key,
            None => return Err(crate::error::Error::MissingSecWebSocketKey),
        };
        let accept_key = derive_accept_key(key.as_bytes());
        let version = req.version();

        tokio::spawn(async move {
            match hyper::upgrade::on(&mut req).await {
                Ok(upgraded) => {
                    let upgraded = TokioIo::new(upgraded);
                    let stream = WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await;

                    let _ = WebSocketServer::on_ws_connection(stream, tx, addr).await;
                }
                Err(e) => log::debug!("upgrade error: {}", e),
            }
        });

        let response = Response::builder()
            .version(version)
            .status(hyper::StatusCode::SWITCHING_PROTOCOLS)
            .header(hyper::header::UPGRADE, "websocket")
            .header(hyper::header::CONNECTION, "upgrade")
            .header("Sec-WebSocket-Accept", &accept_key[..])
            .header("Sec-WebSocket-Protocol", "sip")
            .body(Body::default())
            .expect("Failed to create response");

        Ok(response)
    }

    pub(crate) async fn handle_incoming(self, tx: TransportTx) -> Result<()> {
        loop {
            let (stream, remote_addr) = self.sock.accept().await?;
            let tx = tx.clone();

            println!("Got incoming WebSocket connection from {}", remote_addr);

            // Let's spawn the handling of each connection in a separate task.
            tokio::spawn(async move {
                let io = TokioIo::new(stream);

                let service = service_fn(move |req| WebSocketServer::on_received_request(req, tx.clone(), remote_addr));

                let conn = http1::Builder::new().serve_connection(io, service).with_upgrades();

                if let Err(err) = conn.await {
                    log::error!("failed to serve connection: {err:?}");
                }
            });
        }
    }
}

pub(crate) struct WsStartup {
    addr: SocketAddr,
}

impl WsStartup {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
}
