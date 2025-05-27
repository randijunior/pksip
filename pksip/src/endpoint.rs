#![deny(missing_docs)]
//! SIP Endpoint

use itertools::Itertools;

use crate::headers::{Header, Via};
use crate::message::{Host, HostPort, Response, StatusLine};
use crate::transaction::server::TsxUas;
use crate::transaction::server_inv::TsxUasInv;
use crate::transport::tcp::TcpStartup;
use crate::transport::udp::UdpStartup;
use crate::transport::{
    self, IncomingRequest, IncomingResponse, OutgoingAddr, OutgoingResponse, Transport, TransportLayer,
};
use crate::{find_map_header, SipService};

use crate::{headers::Headers, transaction::TransactionLayer, Result};

use std::net::{IpAddr, SocketAddr};
use std::{io, sync::Arc};

struct Inner {
    /// The transport layer for the endpoint.
    transport: TransportLayer,

    /// The transaction layer for the endpoint.
    transaction: Option<TransactionLayer>,

    /// The name of the endpoint.
    name: String,

    /// The capability header list.
    capabilities: Headers<'static>,

    /// The resolver for DNS lookups.
    resolver: Resolver,

    /// The list of services registered.
    services: Box<[Box<dyn SipService>]>,
}

#[derive(Clone)]
/// The SIP endpoint.
///
/// An endpoint is a logical entity that can send and receive SIP messages,
/// manage transactions, and interact with various SIP services. The endpoint is
/// responsible for handling incoming requests and responses, as well as sending
/// outgoing messages.
pub struct Endpoint(Arc<Inner>);

impl Endpoint {
    /// Returns a builder to create an `Endpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::Builder::new()
    ///     .with_name("My Endpoint")
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Runs the endpoint by processing messages from transport layer.
    ///
    /// This method spawns a new Tokio task that will run indefinitely,
    /// processing incoming SIP messages.
    pub async fn run(self) -> Result<()> {
        tokio::spawn(Box::pin(self.receive_message()))
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Task join error: {}", e)))??;

        Ok(())
    }

    async fn receive_message(self) -> Result<()> {
        self.0.transport.receive_packet(&self).await
    }

    /// Get the endpoint name.
    pub fn get_name(&self) -> &String {
        &self.0.name
    }

    /// Creates a new User Agent Server (UAS) transaction.
    ///
    /// This method initializes an [`TsxUas`] instance, which represents
    /// the server transaction for handling incoming SIP requests that
    /// are not `INVITE` requests.
    pub fn new_uas_tsx(&self, request: &mut IncomingRequest) -> TsxUas {
        TsxUas::new(self, request)
    }

    /// Creates a new User Agent Server (UAS) Invite transaction.
    ///
    /// This method initializes an [`TsxUasInv`] instance, which represents
    /// the server transaction for handling an incoming `INVITE` request.
    pub fn new_uas_inv_tsx(&self, request: &mut IncomingRequest) -> TsxUasInv {
        TsxUasInv::new(self, request)
    }

    /// Respond statelessly an request.
    ///
    /// This method create an response from the incoming request and
    /// sent statelessly, meaning that no `UAS` transaction must be
    /// created for this request.
    pub async fn respond(&self, request: &IncomingRequest<'_>, status_code: i32, reason_phrase: &str) -> Result<()> {
        // No `UAS` transaction must be created for this request.
        assert!(request.tsx.is_none(), "Request already has a transaction");

        let msg = self.new_response(request, status_code, reason_phrase);

        self.send_response(&msg).await
    }

    /// Respond statelessly with headers and body.
    ///
    /// Same as [`Endpoint::respond`] but allows to set additional headers and
    /// body.
    pub async fn respond_with_headers_and_body(
        &self,
        request: &IncomingRequest<'_>,
        status_code: i32,
        reason_phrase: &str,
        mut headers: Headers<'_>,
        body: &[u8],
    ) -> Result<()> {
        // No `UAS` transaction must be created for this request.
        assert!(request.tsx.is_none(), "Request already has a transaction");

        let mut msg = self.new_response(request, status_code, reason_phrase);

        msg.append_headers(&mut headers);

        msg.set_body(body);

        self.send_response(&msg).await
    }

    /// Creates a new SIP response based on an incoming request.
    ///
    /// This method generates a response message with the specified status code
    /// and reason phrase. It also sets the necessary headers from request,
    /// including Call-ID, From, To, CSeq, and Via headers.
    pub fn new_response<'a>(&self, req: &'a IncomingRequest<'a>, code: i32, reason: &'a str) -> OutgoingResponse<'a> {
        // Copy the necessary headers from the request.
        let mut headers = Headers::with_capacity(7);

        // `Via` header.
        let topmost_via = req.req_headers.via.clone();
        let via = req.msg.headers.filter(|h| matches!(h, Header::Via(_))).skip(1);
        headers.push(Header::Via(topmost_via));
        headers.extend(via.cloned());

        // `Record-Route` header.
        let rr = req.msg.headers.filter(|h| matches!(h, Header::RecordRoute(_)));
        headers.extend(rr.cloned());

        // `Call-ID` header.
        headers.push(Header::CallId(req.req_headers.call_id));

        // `From` header.
        let from = find_map_header!(req.msg.headers, From).cloned();
        if let Some(from) = from {
            headers.push(Header::From(from));
        }

        // `To` header.
        let to = find_map_header!(req.msg.headers, To);
        if let Some(to) = to {
            let mut to = to.clone();
            // 8.2.6.2 Headers and Tags
            // The UAS MUST add a tag to the To header field in
            // the response (with the exception of the 100 (Trying)
            // response, in which a tag MAY be present).
            if to.tag().is_none() && code > 100 {
                to.set_tag(req.req_headers.via.branch());
            }
            headers.push(Header::To(to));
        }

        // `CSeq` header.
        let cseq = find_map_header!(req.msg.headers, CSeq).cloned();
        if let Some(cseq) = cseq {
            headers.push(Header::CSeq(cseq));
        }

        let addr = self.get_outbound_addr(&req.req_headers.via, &req.transport);
        let status_line = StatusLine::new(code.into(), reason);

        // Done.
        OutgoingResponse {
            msg: Response {
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
    /// This method encodes the response message and sends it to the
    /// specified address using the appropriate transport layer.
    pub async fn send_response(&self, response: &OutgoingResponse<'_>) -> Result<()> {
        log::debug!(
            "=> Response {} {}",
            response.status_code().into_i32(),
            response.reason()
        );
        let encoded_buf = response.encode()?;
        let encoded_slice = encoded_buf.as_slice();

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
                    format!("Transport not found for {}:{} {}", ip, port, protocol),
                ))?;
                transport.send(encoded_slice, &addr).await?;
                Ok(())
            }
            OutgoingAddr::Addr { addr, ref transport } => {
                transport.send(encoded_slice, &addr).await?;
                Ok(())
            }
        }
    }

    async fn resolve_host_to_ip(&self, host: &Host) -> Result<IpAddr> {
        match host {
            Host::DomainName(domain) => Ok(self.0.resolver.resolve(domain).await?),
            Host::IpAddr(ip) => Ok(*ip),
        }
    }

    // https://datatracker.ietf.org/doc/html/rfc3261#section-18.2.2
    // https://datatracker.ietf.org/doc/html/rfc3581s
    fn get_outbound_addr(&self, via: &Via<'_>, transport: &Transport) -> OutgoingAddr {
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
            let ip = via.received().unwrap();
            let port = via.sent_by().port.unwrap_or(5060);
            let addr = SocketAddr::new(ip, port);

            OutgoingAddr::Addr {
                addr,
                transport: transport.clone(),
            }
        }
    }

    pub(crate) async fn process_response(&self, msg: &mut IncomingResponse<'_>) -> Result<()> {
        log::debug!(
            "<= Response ({} {})",
            msg.msg.status_line.code.into_i32(),
            msg.msg.status_line.reason
        );
        let mut handled = false;
        for service in self.0.services.iter() {
            handled = service.on_incoming_response(self, msg).await?;

            if handled {
                break;
            }
        }

        if !handled {
            log::debug!(
                "Response ({} {}) from /{} was unhandled by any sevice",
                msg.msg.status_line.code.into_i32(),
                msg.msg.status_line.reason,
                msg.packet.addr
            );
        }

        Ok(())
    }

    pub(crate) async fn process_request(&self, msg: &mut IncomingRequest<'_>) -> Result<()> {
        log::debug!("<= Request {} from /{}", msg.method(), msg.addr());

        let handled_by_transaction = match self.0.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_request(msg).await?,
            None => false,
        };

        if handled_by_transaction {
            log::debug!("HANDLED BY TRANSACTION");
            return Ok(());
        }

        // If the request was not handled by the transaction layer, we
        // pass it to the services.
        let mut handled = false;
        for service in self.0.services.iter() {
            handled = service.on_incoming_request(self, msg).await?;
            if handled {
                break;
            }
        }
        if !handled {
            log::debug!(
                "Request ({}, cseq=) from /{} was unhandled by any sevice",
                msg.method(),
                // msg.cseq().cseq(),
                msg.addr()
            );
        }

        Ok(())
    }

    pub(crate) fn get_tsx_layer(&self) -> &TransactionLayer {
        self.0.transaction.as_ref().expect("Transaction layer not set")
    }
    pub(crate) fn transport(&self) -> &TransportLayer {
        &self.0.transport
    }
}

/// Builder for creating a new SIP `Endpoint`.
pub struct Builder {
    name: String,
    resolver: Resolver,
    transport: TransportLayer,
    transaction: Option<TransactionLayer>,
    capabilities: Headers<'static>,
    services: Vec<Box<dyn SipService>>,

    transport_start: Vec<Box<dyn transport::TransportStartup>>,
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct a `Endpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::Builder::new().with_name("My Endpoint").build();
    /// ```
    pub fn new() -> Self {
        Builder {
            transport: TransportLayer::new(),
            name: String::new(),
            capabilities: Headers::new(),
            resolver: Resolver::default(),
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
    /// let endpoint = endpoint::Builder::new().with_name("My Endpoint").build();
    /// ```
    pub fn with_name<T: AsRef<str>>(mut self, s: T) -> Self {
        self.name = s.as_ref().to_string();

        self
    }

    /// Add a new capability to the endpoint.
    pub fn add_capability(mut self, capability: Header<'static>) -> Self {
        self.capabilities.push(capability);

        self
    }

    /// Add a new builder for TCP transport on specified address.
    pub fn with_tcp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(TcpStartup::new(addr)));
        self
    }

    /// Add a new builder for TCP transport on specified address.
    pub fn with_udp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(UdpStartup::new(addr)));
        self
    }

    /// Adds a service to the endpoint.
    ///
    /// This function can be called multiple times to add additional services.
    /// If a service with the same name already exists, the new service will not
    /// be added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl SipService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    /// let endpoint = endpoint::Builder::new().with_service(MyService).build();
    /// ```
    pub fn with_service(mut self, service: impl SipService) -> Self {
        if self.service_exists(service.name()) {
            return self;
        }
        self.services.push(Box::new(service));

        self
    }

    /// Add a collection of services to the endpoint.
    ///
    /// Similar to [`Builder::with_service`], but allows adding multiple
    /// services at once. Unlike `with_service`, this method expects the
    /// services to be passed as trait objects (`Box<dyn SipService>`)
    /// instead of concrete types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl SipService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    ///
    /// struct OtherService;
    ///
    /// impl SipService for OtherService {
    ///     fn name(&self) -> &str {
    ///         "OtherService"
    ///     }
    /// }
    ///
    /// let endpoint = endpoint::Builder::new()
    ///     .with_services([
    ///         Box::new(MyService) as Box<dyn SipService>,
    ///         Box::new(OtherService) as Box<dyn SipService>,
    ///     ])
    ///     .build();
    /// ```
    pub fn with_services<I>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn SipService>>,
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
    pub fn with_transaction_layer(mut self, tsx_layer: TransactionLayer) -> Self {
        self.transaction = Some(tsx_layer);

        self
    }

    /// Finalize the builder into a `Endpoint`.
    pub async fn build(self) -> Endpoint {
        log::trace!("Creating endpoint...");
        log::debug!(
            "Services registered {}",
            format_args!("({})", self.services.iter().map(|s| s.name()).join(", "))
        );

        let endpoint = Endpoint(Arc::new(Inner {
            transaction: self.transaction,
            transport: self.transport,
            name: self.name,
            capabilities: self.capabilities,
            resolver: self.resolver,
            services: self.services.into_boxed_slice(),
        }));

        let tx = endpoint.transport().sender();

        for tp_start in self.transport_start {
            tp_start.start(tx.clone()).await.expect("Failed");
        }

        endpoint
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

use hickory_resolver::{error::ResolveError, lookup_ip::LookupIp};

/// Resolver
pub struct Resolver {
    dns_resolver: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    async fn lookup(&self, host: &str) -> std::result::Result<LookupIp, ResolveError> {
        self.dns_resolver.lookup_ip(host).await
    }
    /// Resolve a single.
    pub async fn resolve(&self, host: &str) -> Result<IpAddr> {
        Ok(self
            .lookup(host)
            .await
            .map_err(|err| io::Error::other(format!("Failed to lookup DNS: {}", err)))?
            .iter()
            .next()
            .unwrap())
    }
    /// Resolve a all.
    pub async fn resolve_all(&self, host: &str) -> Result<Vec<IpAddr>> {
        let result = self
            .lookup(host)
            .await
            .map_err(|err| io::Error::other(format!("Failed to lookup dns: {}", err)))?;

        let addresses = result.iter().collect();

        Ok(addresses)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            dns_resolver: hickory_resolver::AsyncResolver::tokio_from_system_conf()
                .expect("Failed to get DNS resolver"),
        }
    }
}
