#![warn(missing_docs)]
//! SIP SipEndpoint

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use itertools::Itertools;
use util::DnsResolver;

use crate::core::to_take::ToTake;
use crate::header::{Header, Headers, Via};
use crate::message::{Host, HostPort, Response, StatusCode, StatusLine};
use crate::transaction::{ServerInvTransaction, ServerTransaction, Transactions};
use crate::transport::tcp::{TcpFactory, TcpStartup};
use crate::transport::udp::UdpStartup;
use crate::transport::{
    Encode, IncomingRequest, IncomingResponse, OutgoingAddr, OutgoingResponse, TransportRef,
    TransportStartup, Transports,
};
use crate::{transport, EndpointService, Result};

struct Inner {
    /// The transport layer for the endpoint.
    transport: Transports,
    /// The transaction layer for the endpoint.
    transaction: Option<Transactions>,
    /// The name of the endpoint.
    name: String,
    /// The capability header list.
    capabilities: Headers,
    /// The resolver for DNS lookups.
    resolver: DnsResolver,
    /// The list of services registered.
    services: Vec<Box<dyn EndpointService>>,
    // user_agent: UserAgent
}

#[derive(Clone)]
/// The SIP endpoint.
///
/// An endpoint is a logical entity that can send and
/// receive SIP messages, manage transactions, and interact
/// with various SIP services. The endpoint is responsible
/// for handling incoming requests and responses, as well as
/// sending outgoing messages.
pub struct SipEndpoint(Arc<Inner>);

impl SipEndpoint {
    /// Returns a EndpointBuilder to create an `SipEndpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My SipEndpoint")
    ///     .build();
    /// ```
    pub fn builder() -> EndpointBuilder {
        EndpointBuilder::default()
    }

    /// Run with timeout
    pub async fn run_with_timeout(self, timeout: Duration) -> Result<()> {
        let _ = tokio::time::timeout(timeout, self.receive_message()).await;

        Ok(())
    }

    /// Runs the endpoint by processing messages from
    /// transport layer.
    ///
    /// This method spawns a new Tokio task that will run
    /// indefinitely, processing incoming SIP messages.
    pub async fn run(self) -> Result<()> {
        tokio::spawn(Box::pin(self.receive_message()))
            .await
            .map_err(crate::Error::JoinError)?
    }

    async fn receive_message(self) -> Result<()> {
        self.0.transport.handle_events(&self).await
    }

    /// Get the endpoint name.
    pub fn get_name(&self) -> &String {
        &self.0.name
    }

    /// Creates a new Server transaction.
    ///
    /// This method initializes an [`ServerTransaction`]
    /// instance, which represents the server
    /// transaction for handling incoming SIP requests that
    /// are not `INVITE` requests.
    pub fn new_server_transaction(&self, request: &IncomingRequest) -> ServerTransaction {
        ServerTransaction::new(self, request)
    }

    /// Creates a new Server Invite transaction.
    ///
    /// This method initializes an [`ServerInvTransaction`]
    /// instance, which represents the server
    /// transaction for handling an incoming `INVITE`
    /// request.
    pub fn new_inv_server_transaction(&self, request: &IncomingRequest) -> ServerInvTransaction {
        ServerInvTransaction::new(self, request)
    }

    /// Respond statelessly an request.
    ///
    /// This method create an response from the incoming
    /// request and sent statelessly, meaning that no
    /// `UAS` transaction must be created for this
    /// request.
    pub async fn respond(
        &self,
        request: &IncomingRequest,
        status_code: StatusCode,
        reason_phrase: Option<Arc<str>>,
    ) -> Result<()> {
        // No `UAS` transaction must be created for this request.
        assert!(
            request.transaction.is_none(),
            "Request already has a transaction"
        );

        let msg = self.new_response(request, status_code, reason_phrase);

        self.send_response(&msg).await
    }

    /// Creates a new SIP response based on an incoming
    /// request.
    ///
    /// This method generates a response message with the specified status code
    /// and reason phrase. It also sets the necessary headers from request,
    /// including `Call-ID`, `From`, `To`, `CSeq`, `Via` and
    /// `Record-Route` headers.
    pub fn new_response(
        &self,
        request: &IncomingRequest,
        status_code: StatusCode,
        reason_phrase: Option<Arc<str>>,
    ) -> OutgoingResponse {
        // Copy the necessary headers from the request.
        let mut headers = Headers::with_capacity(7);
        let all_hdrs = &request.msg.headers;
        let req_hdrs = &request.request_headers;

        // `Via` header.
        let topmost_via = req_hdrs.via.clone();
        headers.push(Header::Via(topmost_via));
        let other_vias = all_hdrs
            .iter()
            .filter(|h| matches!(h, Header::Via(_)))
            .skip(1);
        headers.extend(other_vias.cloned());

        // `Record-Route` header.
        let rr = all_hdrs
            .iter()
            .filter(|h| matches!(h, Header::RecordRoute(_)));
        headers.extend(rr.cloned());

        // `Call-ID` header.
        headers.push(Header::CallId(req_hdrs.call_id.clone()));

        // `From` header.
        headers.push(Header::From(req_hdrs.from.clone()));

        // `To` header.
        let mut to = req_hdrs.to.clone();
        // 8.2.6.2 Headers and Tags
        // The UAS MUST add a tag to the To header field in
        // the response (with the exception of the 100 (Trying)
        // response, in which a tag MAY be present).
        if to.tag().is_none() && status_code.as_u16() > 100 {
            to.set_tag(req_hdrs.via.branch().map(|s| s.to_string()));
        }
        headers.push(Header::To(to));

        // `CSeq` header.
        headers.push(Header::CSeq(req_hdrs.cseq));

        let addr = self.get_outbound_addr(&req_hdrs.via, &request.transport);
        let reason = match reason_phrase {
            None => status_code.reason(),
            Some(reason) => reason,
        };
        let status_line = StatusLine::new(status_code, reason);

        // Done.
        OutgoingResponse {
            response: Response {
                status_line,
                headers,
                body: None,
            },
            addr,
            buf: None,
        }
    }

    /// Sends a SIP response to the specified address.
    ///
    /// This method encodes the response message and sends
    /// it to the specified address using the
    /// appropriate transport layer.
    pub async fn send_response(&self, response: &OutgoingResponse) -> Result<()> {
        log::debug!(
            "=> Response {} {}",
            response.status_code().as_u16(),
            response.reason()
        );
        let encoded_buf = response.encode()?;

        match response.addr {
            OutgoingAddr::HostPort {
                host: HostPort { ref host, port },
                protocol,
            } => {
                let ip = self.resolve_host_to_ip(host).await?;
                let port = port.unwrap();
                let addr = SocketAddr::new(ip, port);

                // Find the transport for the given address and protocol.
                let transport = self.0.transport.find(addr, protocol);
                let transport = transport.ok_or(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("TransportRef not found for {}:{} {}", ip, port, protocol),
                ))?;
                transport.send(&encoded_buf, &addr).await?;
                Ok(())
            }
            OutgoingAddr::Addr {
                addr,
                ref transport,
            } => {
                transport.send(&encoded_buf, &addr).await?;
                Ok(())
            }
        }
    }

    async fn resolve_host_to_ip(&self, host: &Host) -> Result<IpAddr> {
        match host {
            Host::DomainName(domain) => Ok(self.0.resolver.resolve(domain.as_str()).await?),
            Host::IpAddr(ip) => Ok(*ip),
        }
    }

    // https://datatracker.ietf.org/doc/html/rfc3261#section-18.2.2
    // https://datatracker.ietf.org/doc/html/rfc3581s
    fn get_outbound_addr(&self, via: &Via, transport: &TransportRef) -> OutgoingAddr {
        if transport.reliable() {
            // Tcp, TLS, etc..
            return OutgoingAddr::Addr {
                addr: transport.addr(),
                transport: transport.clone(),
            };
        }

        if let Some(maddr) = via.maddr() {
            let port = via.sent_by().port.unwrap_or(5060);

            OutgoingAddr::HostPort {
                host: HostPort {
                    host: maddr.clone(),
                    port: Some(port),
                },
                protocol: via.transport(),
            }
        } else if let Some(rport) = via.rport() {
            let ip = via.received().unwrap();
            let addr = SocketAddr::new(ip, rport);

            OutgoingAddr::Addr {
                addr,
                transport: transport.clone(),
            }
        } else {
            let ip = via
                .received()
                .expect("Missing received parameter on 'Via' header");
            let port = via.sent_by().port.unwrap_or(5060);
            let addr = SocketAddr::new(ip, port);

            OutgoingAddr::Addr {
                addr,
                transport: transport.clone(),
            }
        }
    }

    pub(crate) async fn process_response(&self, msg: &mut Option<IncomingResponse>) -> Result<()> {
        {
            let msg = msg.as_ref().unwrap();
            log::debug!(
                "<= Response ({} {})",
                msg.response.status_line.code.as_u16(),
                msg.response.status_line.reason
            );
        }

        let handled_by_tsx_layer = match self.0.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_response(msg.as_ref().unwrap()).await?,
            None => false,
        };

        if handled_by_tsx_layer {
            return Ok(());
        }

        for service in self.0.services.iter() {
            service.on_incoming_response(self, msg).await?;

            if msg.is_none() {
                break;
            }
        }

        if let Some(msg) = msg {
            log::debug!(
                "Response ({} {}) from /{} was unhandled by any sevice",
                msg.response.status_line.code.as_u16(),
                msg.response.status_line.reason,
                msg.packet.addr
            );
        }

        Ok(())
    }

    pub(crate) async fn process_request(&self, msg: &mut Option<IncomingRequest>) -> Result<()> {
        {
            let msg = msg.as_ref().unwrap();
            log::debug!("<= Request {} from /{}", msg.method(), msg.addr());
        }

        let handled_by_tsx_layer = match self.0.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_request(msg.as_ref().unwrap()).await?,
            None => false,
        };

        if handled_by_tsx_layer {
            return Ok(());
        }

        // If the request was not handled by the transaction layer,
        // we pass it to the services.
        for service in self.0.services.iter() {
            service.on_incoming_request(self, ToTake::new(msg)).await?;
            if msg.is_none() {
                break;
            }
        }
        if let Some(msg) = msg {
            log::debug!(
                "Request ({}, cseq={}) from /{} was unhandled by any sevice",
                msg.method(),
                msg.request_headers.cseq.cseq,
                msg.addr()
            );
        }

        Ok(())
    }

    pub(crate) fn transactions(&self) -> &Transactions {
        self.0
            .transaction
            .as_ref()
            .expect("Transaction layer not set")
    }

    pub(crate) fn transports(&self) -> &Transports {
        &self.0.transport
    }
}

/// EndpointBuilder for creating a new SIP `SipEndpoint`.
pub struct EndpointBuilder {
    name: String,
    resolver: DnsResolver,
    factories: Vec<Box<dyn transport::Factory>>,
    transaction: Option<Transactions>,
    capabilities: Headers,
    services: Vec<Box<dyn EndpointService>>,
    transport_start: Vec<Box<dyn TransportStartup>>,
}

impl EndpointBuilder {
    /// Creates a new default instance of `EndpointBuilder` to
    /// construct a `SipEndpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My SipEndpoint")
    ///     .build();
    /// ```
    pub fn new() -> Self {
        EndpointBuilder {
            factories: Vec::new(),
            name: String::new(),
            capabilities: Headers::new(),
            resolver: DnsResolver::default(),
            services: vec![],
            transaction: None,
            transport_start: vec![],
        }
    }

    /// Sets the endpoint name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My SipEndpoint")
    ///     .build();
    /// ```
    pub fn with_name<T: AsRef<str>>(mut self, s: T) -> Self {
        self.name = s.as_ref().to_string();

        self
    }

    /// Add a new capability to the endpoint.
    pub fn add_capability(mut self, capability: Header) -> Self {
        self.capabilities.push(capability);

        self
    }

    /// Add a new EndpointBuilder for TCP transport on specified
    /// address.
    pub fn with_tcp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(TcpStartup::new(addr)));
        self.factories.push(Box::new(TcpFactory));
        self
    }

    /// Add a new EndpointBuilder for TCP transport on specified
    /// address.
    pub fn with_udp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(UdpStartup::new(addr)));
        self
    }

    /// Adds a service to the endpoint.
    ///
    /// This function can be called multiple times to add
    /// additional services. If a service with the same
    /// name already exists, the new service will not be
    /// added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl EndpointService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_service(MyService)
    ///     .build();
    /// ```
    pub fn with_service(mut self, service: impl EndpointService) -> Self {
        if self.service_exists(service.name()) {
            return self;
        }
        self.services.push(Box::new(service));

        self
    }

    /// Add a collection of services to the endpoint.
    ///
    /// Similar to [`EndpointBuilder::with_service`], but allows
    /// adding multiple services at once. Unlike
    /// `with_service`, this method expects the services
    /// to be passed as trait objects (`Box<dyn
    /// EndpointService>`) instead of concrete types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl EndpointService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    ///
    /// struct OtherService;
    ///
    /// impl EndpointService for OtherService {
    ///     fn name(&self) -> &str {
    ///         "OtherService"
    ///     }
    /// }
    ///
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_services([
    ///         Box::new(MyService) as Box<dyn EndpointService>,
    ///         Box::new(OtherService) as Box<dyn EndpointService>,
    ///     ])
    ///     .build();
    /// ```
    pub fn with_services<I>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn EndpointService>>,
    {
        for service in services {
            if self.service_exists(service.name()) {
                continue;
            }
            self.services.push(service);
        }

        self
    }

    fn service_exists(&self, name: &str) -> bool {
        let exists = self.services.iter().any(|s| s.name() == name);
        if exists {
            log::warn!("Service with name '{}' already exists", name);
        }
        exists
    }

    /// Sets the transaction layer.
    pub fn with_transaction(mut self, tsx_layer: Transactions) -> Self {
        self.transaction = Some(tsx_layer);

        self
    }

    /// Finalize the EndpointBuilder into a `SipEndpoint`.
    pub async fn build(self) -> SipEndpoint {
        log::trace!("Creating endpoint...");
        log::debug!(
            "Services registered {}",
            format_args!("({})", self.services.iter().map(|s| s.name()).join(", "))
        );

        let endpoint = SipEndpoint(Arc::new(Inner {
            transaction: self.transaction,
            transport: Transports::new(self.factories),
            name: self.name,
            capabilities: self.capabilities,
            resolver: self.resolver,
            services: self.services,
        }));

        let tx = endpoint.transports().sender();

        for tp_start in self.transport_start {
            tp_start.start(tx.clone()).await.expect("Failed");
        }

        endpoint
    }
}

impl Default for EndpointBuilder {
    fn default() -> Self {
        Self::new()
    }
}
