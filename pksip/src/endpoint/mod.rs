#![deny(missing_docs)]
//! SIP Endpoint
//!

pub mod builder;
pub mod service;

mod resolver;

pub use builder::Builder;

use crate::endpoint::resolver::Resolver;
use crate::headers::{Header, Via};
use crate::message::{Host, HostPort, Response, StatusLine};
use crate::transaction::server::ServerTransaction;
use crate::transaction::inv_server::InvServerTransaction;

use crate::transport::{IncomingRequest, IncomingResponse, OutgoingAddr, OutgoingResponse, ToBytes, Transport, TransportLayer};
use crate::SipService;
use crate::{headers::Headers, transaction::TransactionLayer, Result};

use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
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

    /// Run with timeout
    pub async fn run_with_timeout(self, timeout: Duration) -> Result<()> {
        let _ = tokio::time::timeout(timeout, self.receive_message()).await;

        Ok(())
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
        self.0.transport.handle_events(&self).await
    }

    /// Get the endpoint name.
    pub fn get_name(&self) -> &String {
        &self.0.name
    }

    /// Creates a new User Agent Server (UAS) transaction.
    ///
    /// This method initializes an [`ServerTransaction`] instance, which represents
    /// the server transaction for handling incoming SIP requests that
    /// are not `INVITE` requests.
    pub fn new_uas_tsx(&self, request: &mut IncomingRequest) -> ServerTransaction {
        ServerTransaction::new(self, request)
    }

    /// Creates a new User Agent Server (UAS) Invite transaction.
    ///
    /// This method initializes an [`InvServerTransaction`] instance, which represents
    /// the server transaction for handling an incoming `INVITE` request.
    pub fn new_uas_inv_tsx(&self, request: &mut IncomingRequest<'_>) -> InvServerTransaction {
        InvServerTransaction::new(self, request)
    }

    /// Respond statelessly an request.
    ///
    /// This method create an response from the incoming request and
    /// sent statelessly, meaning that no `UAS` transaction must be
    /// created for this request.
    pub async fn respond(&self, request: &IncomingRequest<'_>, status_code: i32, reason_phrase: &str) -> Result<()> {
        // No `UAS` transaction must be created for this request.
        assert!(request.transaction.is_none(), "Request already has a transaction");

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
        assert!(request.transaction.is_none(), "Request already has a transaction");

        let mut msg = self.new_response(request, status_code, reason_phrase);

        msg.append_headers(&mut headers);

        msg.set_body(body);

        self.send_response(&msg).await
    }

    /// Creates a new SIP response based on an incoming request.
    ///
    /// This method generates a response message with the specified status code
    /// and reason phrase. It also sets the necessary headers from request,
    /// including `Call-ID`, `From`, `To`, `CSeq`, `Via` and `Record-Route` headers.
    pub fn new_response<'a>(&self, req: &'a IncomingRequest<'a>, code: i32, reason: &'a str) -> OutgoingResponse<'a> {
        // Copy the necessary headers from the request.
        let mut headers = Headers::with_capacity(7);
        let msg_headers = &req.request.headers;

        // `Via` header.
        let topmost_via = req.request_headers.via.clone();
        let via = msg_headers.iter().filter(|h| matches!(h, Header::Via(_))).skip(1);
        headers.push(Header::Via(topmost_via));
        headers.extend(via.cloned());

        // `Record-Route` header.
        let rr = msg_headers.iter().filter(|h| matches!(h, Header::RecordRoute(_)));
        headers.extend(rr.cloned());

        // `Call-ID` header.
        headers.push(Header::CallId(req.request_headers.call_id.clone()));

        // `From` header.
        let from = msg_headers
            .iter()
            .find_map(|h| if let Header::From(from) = h { Some(from) } else { None })
            .cloned();

        if let Some(from) = from {
            headers.push(Header::From(from));
        }

        // `To` header.
        let to = msg_headers.iter().find_map(|h| match h {
            Header::To(e) => Some(e),
            _ => None,
        });

        if let Some(to) = to {
            let mut to = to.clone();
            // 8.2.6.2 Headers and Tags
            // The UAS MUST add a tag to the To header field in
            // the response (with the exception of the 100 (Trying)
            // response, in which a tag MAY be present).
            if to.tag().is_none() && code > 100 {
                to.set_tag(req.request_headers.via.branch());
            }
            headers.push(Header::To(to));
        }

        // `CSeq` header.
        let cseq = msg_headers.iter().find_map(|h| match h {
            Header::CSeq(e) => Some(e),
            _ => None,
        });

        if let Some(cseq) = cseq {
            headers.push(Header::CSeq(*cseq));
        }

        let addr = self.get_outbound_addr(&req.request_headers.via, &req.transport);
        let status_line = StatusLine::new(code.into(), reason);

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
    /// This method encodes the response message and sends it to the
    /// specified address using the appropriate transport layer.
    pub async fn send_response(&self, response: &OutgoingResponse<'_>) -> Result<()> {
        log::debug!(
            "=> Response {} {}",
            response.status_code().into_i32(),
            response.reason()
        );
        let encoded_buf = response.to_bytes()?;

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
                    format!("Arc<dyn Transport> not found for {}:{} {}", ip, port, protocol),
                ))?;
                transport.send(&encoded_buf, &addr).await?;
                Ok(())
            }
            OutgoingAddr::Addr { addr, ref transport } => {
                transport.send(&encoded_buf, &addr).await?;
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
    fn get_outbound_addr(&self, via: &Via<'_>, transport: &Arc<dyn Transport>) -> OutgoingAddr {
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
            let ip = via.received().expect("Missing received parameter on 'Via' header");
            let port = via.sent_by().port.unwrap_or(5060);
            let addr = SocketAddr::new(ip, port);

            OutgoingAddr::Addr {
                addr,
                transport: transport.clone(),
            }
        }
    }

    pub(crate) async fn process_response(&self, msg: &mut Option<IncomingResponse<'_>>) -> Result<()> {
        {
            let msg = msg.as_ref().unwrap();
            log::debug!(
                "<= Response ({} {})",
                msg.response.status_line.code.into_i32(),
                msg.response.status_line.reason
            );
        }

        let handled_by_transaction_layer = match self.0.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_response(msg.as_ref().unwrap()).await?,
            None => false,
        };

        if handled_by_transaction_layer {
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
                msg.response.status_line.code.into_i32(),
                msg.response.status_line.reason,
                msg.packet.addr
            );
        }

        Ok(())
    }

    pub(crate) async fn process_request(&self, msg: &mut Option<IncomingRequest<'_>>) -> Result<()> {
        {
            let msg = msg.as_ref().unwrap();
            log::debug!("<= Request {} from /{}", msg.method(), msg.addr());
        }

        let handled_by_transaction_layer = match self.0.transaction {
            Some(ref tsx_layer) => tsx_layer.handle_request(msg.as_ref().unwrap()).await?,
            None => false,
        };

        if handled_by_transaction_layer {
            return Ok(());
        }

        // If the request was not handled by the transaction layer, we
        // pass it to the services.
        for service in self.0.services.iter() {
            service.on_incoming_request(self, msg).await?;
            if msg.is_none() {
                break;
            }
        }
        if let Some(msg) = msg {
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
