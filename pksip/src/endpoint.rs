#![warn(missing_docs)]
//! SIP Endpoint

use std::{
    borrow::Cow,
    io,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

pub use builder::EndpointBuilder;
use bytes::Bytes;
pub use service::EndpointHandler;
use tokio::net::ToSocketAddrs;
use util::DnsResolver;

use crate::{
    Result, SipMethod,
    error::Error,
    find_map_header,
    headers::{CSeq, CallId, Contact, From, Header, Headers, To, Via},
    message::{
        DomainName, Host, ReasonPhrase, Request, RequestLine, Response, SipMessageBody, SipUri, StatusCode, StatusLine, Uri, UriBuilder
    },
    transaction::{ServerInviteTx, ServerNonInviteTx, ServerTx, manager::TransactionLayer},
    transport::{
        Encode, IncomingMessage, IncomingRequest, IncomingResponse, MandatoryHeaders,
        OutgoingMessage, OutgoingMessageInfo, OutgoingRequest,
        OutgoingResponse, SipTransport, Transport, TransportKey, TransportManager,
        TransportMessage,
        tcp::{TcpFactory, TcpListener},
        udp::UdpTransport,
        websocket::WebSocketListener,
    },
};

mod builder;
mod service;

struct EndpointInner {
    /// The transport layer for the endpoint.
    transport: TransportManager,
    /// The transaction layer for the endpoint.
    transaction: Option<TransactionLayer>,
    /// The name of the endpoint.
    name: String,
    /// The capability header list.
    capabilities: Headers,
    /// The resolver for DNS lookups.
    resolver: DnsResolver,
    /// The list of services registered.
    handlers: Vec<Box<dyn EndpointHandler>>,
    // user_agent: UserAgent
}

#[derive(Clone)]
struct EndpointRef(Arc<EndpointInner>);

impl std::ops::Deref for EndpointRef {
    type Target = EndpointInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
/// A SIP endpoint.
///
/// An endpoint is a logical entity that can send and receive SIP messages, manage
/// TransactionLayer, and interact with various SIP services. The endpoint is responsible
/// for handling incoming requests and responses, as well as sending outgoing
/// messages.
pub struct Endpoint {
    inner: EndpointRef,
}

impl Endpoint {
    /// Returns a EndpointBuilder to create an `Endpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My Endpoint")
    ///     .build();
    /// ```
    pub fn builder() -> EndpointBuilder {
        EndpointBuilder::default()
    }

    /// Get the endpoint name.
    pub fn get_name(&self) -> &String {
        &self.inner.name
    }

    pub(crate) async fn determine_logical_target(message: &Request) {}

    /// Create request.
    pub async fn send_request(&self, request: &OutgoingRequest) -> Result<()> {
        todo!()
    }

    pub async fn start_udp<A: ToSocketAddrs>(&self, addr: A) -> Result<()> {
        let udp = UdpTransport::bind(addr).await?;
        log::info!("UDP transport bound to: {}", udp.local_addr());
        self.transports()
            .register_transport(Transport::new(udp.clone()))?;
        tokio::spawn(udp.receive(self.clone()));
        Ok(())
    }

    pub async fn start_tcp(&self, addr: SocketAddr) -> Result<()> {
        let tcp = TcpListener::bind(addr).await?;
        log::info!("Listening (TCP) on: {}", addr);
        // let factory = TcpFactory::new();
        // // self.register_factory(factory);
        tokio::spawn(tcp.accept_clients(self.clone()));
        Ok(())
    }

    pub async fn start_ws(&self, addr: SocketAddr) -> Result<()> {
        let ws = WebSocketListener::bind(addr).await?;
        log::info!("Listening (WS) on: {}", ws.local_addr());
        // let factory = WsFactory::new(self.clone());
        // self.register_factory(factory);
        tokio::spawn(ws.accept_clients(self.clone()));
        Ok(())
    }

    pub(crate) fn receive_transport_message(&self, message: TransportMessage) {
        tokio::spawn(self.clone().process_transport_message(message));
    }

    async fn process_transport_message(self, message: TransportMessage) -> Result<()> {
        match message.parse() {
            Ok(IncomingMessage::Request(request)) => {
                self.process_request(&request).await?;
            }
            Ok(IncomingMessage::Response(response)) => {
                self.process_response(&mut Some(response)).await?;
            }
            Err(err) => log::error!("ERR = {:#?}", err),
        }

        Ok(())
    }

    pub(crate) async fn dns_lookup(&self, domain: &DomainName) -> Result<IpAddr> {
        Ok(self.inner.resolver.resolve(domain.as_str()).await?)
    }

    pub(crate) fn dns_resolver(&self) -> &DnsResolver {
        &self.inner.resolver
    }
    

    pub(crate) async fn lookup_address(&self, host: &Host) -> Result<IpAddr> {
        match host {
            Host::DomainName(domain) => self.dns_lookup(domain).await,
            Host::IpAddr(ip) => Ok(*ip),
        }
    }

    // https://datatracker.ietf.org/doc/html/rfc3261#section-18.2.2
    // https://datatracker.ietf.org/doc/html/rfc3581s
    async fn get_outbound_addr(
        &self,
        via: &Via,
        transport: &Transport,
    ) -> Result<(SocketAddr, Transport)> {
        if transport.is_reliable() {
            // Tcp, TLS, etc..
            return Ok((transport.remote_addr().unwrap(), transport.clone()));
        }

        if let Some(maddr) = &via.maddr {
            let port = via.sent_by.port.unwrap_or(5060);
            let ip = self.lookup_address(maddr).await?;
            let addr = SocketAddr::new(ip, port);

            return Ok((addr, transport.clone()));
        } else if let Some(rport) = via.rport {
            let ip = via.received.unwrap();
            let addr = SocketAddr::new(ip, rport);
            return Ok((addr, transport.clone()));
        } else {
            let ip = via
                .received
                .expect("Missing received parameter on 'Via' header");
            let port = via.sent_by.port.unwrap_or(5060);
            let addr = SocketAddr::new(ip, port);
            return Ok((addr, transport.clone()));
        }
    }

    pub(crate) async fn process_response(&self, msg: &mut Option<IncomingResponse>) -> Result<()> {
        {
            let msg = msg.as_ref().unwrap();
            log::debug!(
                "<= Response ({} {})",
                msg.message.status_line.code.as_u16(),
                msg.message.status_line.reason.phrase_str()
            );
        }

        let handled_by_tsx_layer = match self.inner.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_response(msg.as_ref().unwrap()).await?,
            None => false,
        };

        if handled_by_tsx_layer {
            return Ok(());
        }

        for service in self.inner.handlers.iter() {
            service.on_incoming_response(self, msg).await?;

            if msg.is_none() {
                break;
            }
        }

        if let Some(msg) = msg {
            log::debug!(
                "Response ({} {}) from /{} was unhandled by any sevice",
                msg.message.status_line.code.as_u16(),
                msg.message.status_line.reason.phrase_str(),
                msg.info.received_packet.packet.source
            );
        }

        Ok(())
    }

    pub(crate) async fn process_request(&self, msg: &IncomingRequest) -> Result<()> {
        log::debug!(
            "<= Request {} from /{}",
            msg.message.method(),
            msg.info.received_packet.packet.source
        );
        let handled_by_tsx_layer = match self.inner.transaction {
            Some(ref tsx_layer) => tsx_layer.on_request(msg).await?,
            None => false,
        };

        if handled_by_tsx_layer {
            return Ok(());
        }

        // If the request was not handled by the transaction layer,
        // we pass it to the handlers.
        let mut response: Option<EndpointResponse> = None;
        for service in self.inner.handlers.iter() {
            response = service.on_request(msg).await;
            if response.is_some() {
                break;
            }
        }
        if let Some(response) = response {
            match response.kind {
                ResponseKind::Stateless => {
                    log::debug!(
                        "=> Response {} {}",
                        response.message.status_line.code.as_u16(),
                        response.message.status_line.reason.phrase_str()
                    );
                    let info = &msg.info;
                    let (destination, transport) = self
                        .get_outbound_addr(
                            &info.mandatory_headers.via,
                            &info.received_packet.transport,
                        )
                        .await?;
                    let encoded = response.response.encode()?;

                    transport.send_msg(&encoded, &destination).await?;
                }
                ResponseKind::Stateful => {
                    // server_tx.respond(response).await?;
                }
            }
        } else {
            log::debug!(
                "Request ({}, cseq={}) from /{} was unhandled by any handler",
                msg.message.method(),
                msg.info.mandatory_headers.cseq.cseq,
                msg.info.received_packet.packet.source
            );
        }

        Ok(())
    }

    pub(crate) fn transactions(&self) -> &TransactionLayer {
        self.inner
            .transaction
            .as_ref()
            .expect("Transaction layer not set")
    }

    pub(crate) fn transports(&self) -> &TransportManager {
        &self.inner.transport
    }
}

pub enum ResponseKind {
    Stateless,
    Stateful,
}

pub struct EndpointResponse {
    kind: ResponseKind,
    response: OutgoingResponse,
}

impl EndpointResponse {
    pub fn stateless(
        request: &IncomingRequest,
        status_code: StatusCode,
        reason_phrase: Option<ReasonPhrase>,
    ) -> Self {
        Self {
            kind: ResponseKind::Stateless,
            response: request.new_response(status_code, reason_phrase),
        }
    }

    pub fn stateful(
        request: &IncomingRequest,
        status_code: StatusCode,
        reason_phrase: Option<ReasonPhrase>,
    ) -> Self {
        Self {
            kind: ResponseKind::Stateful,
            response: request.new_response(status_code, reason_phrase),
        }
    }
}

impl std::ops::Deref for EndpointResponse {
    type Target = OutgoingResponse;

    fn deref(&self) -> &Self::Target {
        &self.response
    }
}
impl std::ops::DerefMut for EndpointResponse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.response
    }
}
