use tokio::sync::mpsc::{self, Receiver};

use crate::message::{SipMethod, StatusLine};
use crate::resolver::HostPortInfo;
use crate::transaction::server::inv::UasInvTsx;
use crate::transaction::server::non_inv::UasTsx;
use crate::transport::{
    IncomingMessage, IncomingRequest, OutgoingInfo, OutgoingResponse,
    Transport, TransportLayer,
};
use crate::{
    headers::{Headers, Via},
    message::{SipResponse, StatusCode},
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transaction::TransactionLayer,
};
use std::{io, net::SocketAddr, ops::Deref, sync::Arc};

pub struct Inner {
    transport: TransportLayer,
    pub(crate) transaction: TransactionLayer,
    name: String,
    capabilities: Headers,
    dns_resolver: Resolver,
    sender: tokio::sync::mpsc::Sender<IncomingMessage>,
}

#[derive(Clone)]
pub struct Endpoint(Arc<Inner>);

impl Deref for Endpoint {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Endpoint {
    /// Runs the endpoint by processing messages from transport layer.
    ///
    /// This method spawns a new Tokio task that will run indefinitely,
    /// processing incoming SIP messages.
    pub async fn run(self) -> io::Result<()> {
        tokio::spawn(Box::pin(self.receive_message())).await?
    }

    async fn receive_message(self) -> io::Result<()> {
        log::info!("Starting endpoint worker thread...");
        self.transport.recv_packet(&self).await
    }

    pub(crate) async fn on_transport_msg(
        &self,
        msg: IncomingMessage,
    ) -> io::Result<()> {
        self.sender.send(msg).await.unwrap();
        Ok(())
    }

    /// Creates a new User Agent Server (UAS) transaction.
    ///
    /// This method initializes an `UasInvTsx` instance, which represents the 
    /// server transaction for handling incoming SIP requests that are not INVITE requests.
    pub fn create_uas_tsx(
        &self,
        request: &IncomingRequest,
    ) -> UasTsx {
        UasTsx::new(self, request)
    }

    /// Creates a new User Agent Server (UAS) transaction for an INVITE request.
    ///
    /// This method initializes an `UasInvTsx` instance, which represents  
    /// the server transaction for handling an incoming INVITE request.
    pub fn create_uas_inv_tsx(
        &self,
        request: &IncomingRequest,
    ) -> UasInvTsx {
        UasInvTsx::new(self, request)
    }

    pub async fn respond(
        &self,
        msg: IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<()> {
        let response = self.new_response(msg, st_line).await?;
        let buf = response.into_buffer()?;
        let addr = &response.info.addr;

        response.info.transport.send(&buf, addr).await?;

        Ok(())
    }

    pub async fn new_response(
        &self,
        mut req: IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<OutgoingResponse> {
        let mut hdrs = req.msg.req_headers.take().unwrap();
        let topmost_via = &mut hdrs.via;

        let info = if topmost_via.maddr.is_some()
            || topmost_via.rport.is_some()
        {
            self.get_outgoing_info(
                topmost_via,
                req.transport(),
                req.packet().addr(),
            )
            .await?
        } else {
            OutgoingInfo {
                addr: req.packet().addr,
                transport: req.transport().clone(),
            }
        };

        if hdrs.to.tag.is_none() && st_line.code > StatusCode::Trying
        {
            hdrs.to.tag = topmost_via.branch.as_ref().cloned();
        }
        let msg = SipResponse::new(st_line, Headers::default(), None);

        Ok(OutgoingResponse {
            hdrs,
            info,
            msg,
            buf: None,
        })
    }

    // follow SIP RFC 3261 in section 18.2.2
    pub async fn get_outgoing_info(
        &self,
        via: &Via,
        transport: &Transport,
        src_addr: &SocketAddr,
    ) -> io::Result<OutgoingInfo> {
        if transport.reliable() {
            // Tcp, TLS, etc..
            todo!()
        } else if let Some(maddr) = &via.maddr {
            let port = via.sent_by.port.unwrap_or(5060);
            let addresses = self
                .dns_resolver
                .resolve(HostPortInfo {
                    host: maddr,
                    protocol: transport.protocol(),
                    port,
                })
                .await?;
            let ServerAddress { addr, protocol } = addresses[0];
            // Find transport
            //TODO: the transport transport must create a transport if it cannot find
            let transport = self
                .transport
                .find(addr, protocol)
                .ok_or(io::Error::other(
                    "Coun'd not find a suitable transport!",
                ))?;

            Ok(OutgoingInfo { addr, transport })
        } else if let Some(rport) = via.rport {
            // MUST use the "received" and "rport" parameter.
            let addr = via.received.unwrap_or(src_addr.ip());
            let addr = SocketAddr::new(addr, rport);
            Ok(OutgoingInfo {
                addr,
                transport: transport.clone(),
            })
        } else {
            Ok(OutgoingInfo {
                addr: *src_addr,
                transport: transport.clone(),
            })
        }
    }

    pub fn get_name(&self) -> &String {
        &self.0.name
    }
}

pub struct EndpointBuilder {
    name: String,
    dns_resolver: Resolver,
    transport: TransportLayer,
    transaction: TransactionLayer,
    // Accept, Allow, Supported
    capabilities: Headers,
    services: Vec<Box<dyn SipService>>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder {
            transport: TransportLayer::new(),
            name: String::new(),
            capabilities: Headers::new(),
            dns_resolver: Resolver::default(),
            services: vec![],
            transaction: TransactionLayer::default(),
        }
    }

    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.transport.add(transport);

        self
    }

    fn service_exists(&self, name: &str) -> bool {
        self.services.iter().any(|s| s.name() == name)
    }

    pub fn with_service(mut self, service: impl SipService) -> Self {
        if self.service_exists(service.name()) {
            log::warn!(
                "Service with name '{}' already exists",
                service.name()
            );
            return self;
        }
        self.services.push(Box::new(service));

        self
    }

    pub fn build(self) -> Endpoint {
        log::info!("Creating endpoint...");
        for name in self.services.iter() {
            log::info!("Service {:?} registered", name.name());
        }
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let endpoint = Endpoint(Arc::new(Inner {
            transaction: self.transaction,
            transport: self.transport,
            name: self.name,
            capabilities: self.capabilities,
            dns_resolver: self.dns_resolver,
            sender: tx,
        }));
        let p = MessageProcessor {
            services: self.services,
            receiver: rx,
        };
        tokio::spawn(Box::pin(p.process_message(endpoint.clone())));

        endpoint
    }
}

pub struct MessageProcessor {
    services: Vec<Box<dyn SipService>>,
    receiver: Receiver<IncomingMessage>,
}

impl MessageProcessor {
    pub async fn process_message(mut self, endpoint: Endpoint) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                IncomingMessage::Request(req) => {
                    log::trace!(
                        "Received [Request method={}] from /{}",
                        req.method(),
                        req.info.packet().addr
                    );
                    let mut msg = Some(req);
                    for service in self.services.iter_mut() {
                        if let Err(_err) = service
                            .on_request(&endpoint, &mut msg)
                            .await
                        {
                            break;
                        }

                        if msg.is_none() {
                            break;
                        }
                    }
                }
                IncomingMessage::Response(res) => {
                    log::trace!(
                        "Received [Response code={}, phrase={}] from /{}",
                        res.msg.st_line.code.into_u32(),
                        res.msg.st_line.rphrase,
                        res.info.packet().addr
                    );
                    let mut msg = Some(res);
                    for service in self.services.iter_mut() {
                        if let Err(_err) = service
                            .on_response(&endpoint, &mut msg)
                            .await
                        {
                            break;
                        }

                        if msg.is_none() {
                            break;
                        }
                    }
                }
            }
        }
    }
}
