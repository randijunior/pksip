use tokio::spawn;

use crate::message::{SipMethod, StatusLine};
use crate::resolver::HostPortInfo;
use crate::service::Request;
use crate::transport::{
    IncomingInfo, IncomingRequest, IncomingResponse, OutgoingInfo,
    OutgoingResponse, Transport, TransportLayer,
};
use crate::{
    headers::{Headers, Via},
    message::{
        HostPort, SipMessage, SipResponse, SipUri, StatusCode, UriBuilder,
    },
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transaction::TransactionLayer,
};
use std::{io, net::SocketAddr, ops::Deref, sync::Arc};

type TransportMessage = (IncomingInfo, SipMessage);

pub struct EndpointBuilder {
    name: String,
    dns_resolver: Resolver,
    transport_layer: TransportLayer,
    transaction_layer: TransactionLayer,
    // Accept, Allow, Supported
    capabilities: Headers,
    services: Vec<Box<dyn SipService>>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder {
            transport_layer: TransportLayer::new(),
            name: String::new(),
            capabilities: Headers::new(),
            dns_resolver: Resolver::default(),
            services: vec![],
            transaction_layer: TransactionLayer::default(),
        }
    }

    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.transport_layer.add(transport);

        self
    }

    pub fn with_service(mut self, service: impl SipService) -> Self {
        if self
            .services
            .iter()
            .find(|s| s.name() == service.name())
            .is_some()
        {
            log::warn!("Service with name '{}' already exists", service.name());
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
        Endpoint(Arc::new(Inner {
            transaction_layer: self.transaction_layer,
            transport_layer: self.transport_layer,
            name: self.name,
            capabilities: self.capabilities,
            dns_resolver: self.dns_resolver,
            services: self.services,
        }))
    }
}
pub struct Inner {
    transport_layer: TransportLayer,
    transaction_layer: TransactionLayer,
    name: String,
    capabilities: Headers,
    dns_resolver: Resolver,
    services: Vec<Box<dyn SipService>>,
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
    pub async fn run(self) -> io::Result<()> {
        let rx = self.transport_layer.initialize();
        tokio::spawn(Box::pin(async move {
            self.transport_layer.recv_packet(rx, &self).await
        })).await?
    }

    pub async fn handle_incoming(
        &self,
        msg: TransportMessage,
    ) -> io::Result<()> {
        tokio::spawn(self.clone().process_message(msg));
        Ok(())
    }

    fn tsx_respond(
        &self,
        msg: IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<()> {
        todo!()
    }

    pub async fn respond(
        &self,
        mut msg: IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<()> {
        let response = self.new_response(&mut msg, st_line).await?;
        let buf = response.into_buffer()?;
        let addr = msg.info.packet().addr;

        msg.info.transport().send(&buf, addr).await?;

        Ok(())
    }

    async fn process_message(self, msg: TransportMessage) -> io::Result<()> {
        let (info, msg) = msg;
        match msg {
            SipMessage::Request(msg) => {
                let req = IncomingRequest::new(msg, info);
                self.receive_request(req).await
            }
            SipMessage::Response(msg) => {
                let res = IncomingResponse::new(msg, info);
                self.receive_response(res).await
            }
        }
    }

    pub async fn new_response(
        &self,
        req: &mut IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<OutgoingResponse> {
        let mut hdrs = req.msg.req_headers.take().unwrap();
        let topmost_via = &mut hdrs.via[0];

        let info = self
            .get_outgoing_info(
                topmost_via,
                req.transport(),
                req.packet().addr(),
            )
            .await?;

        topmost_via.received = Some(req.info.packet().addr().ip());
        if hdrs.to.tag.is_none() && st_line.code > StatusCode::Trying {
            hdrs.to.tag = topmost_via.branch.as_ref().map(|s| s.clone());
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
        src_addr: SocketAddr,
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
            //TODO: the transport transport_layer must create a transport if it cannot find
            let transport = self.transport_layer.find(addr, protocol).ok_or(
                io::Error::other("Coun'd not find a suitable transport!"),
            )?;

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
                addr: src_addr,
                transport: transport.clone(),
            })
        }
    }

    pub async fn create_uas_tsx(
        &self,
        request: &mut IncomingRequest,
    ) -> io::Result<()> {
        if request.is_method(&SipMethod::Ack)
            || request.is_method(&SipMethod::Cancel)
        {
            return Err(io::Error::other(
                "ACK and CANCEL cannot create a transaction",
            ));
        }

        let drop_notifier =
            self.transaction_layer.create_uas_tsx(self, request).await?;
        let endpoint = self.clone();
        let key = request.tsx_key.as_ref().unwrap().clone();
        tokio::spawn(async move {
            let _ = drop_notifier.await;
            endpoint.transaction_layer.remove(&key)
        });

        Ok(())
    }

    pub async fn receive_request(
        &self,
        msg: IncomingRequest,
    ) -> io::Result<()> {
        // Create the server transaction.
        // self.create_uas_tsx(&mut msg).await?;
        // new_incoming
        let mut request = Request {
            endpoint: self,
            msg: msg.into(),
        };
        for service in self.services.iter() {
            if let Err(_err) = service.on_request(&mut request).await {
                break;
            }

            if request.msg.is_none() {
                break;
            }
        }

        Ok(())
    }

    pub async fn receive_response(
        &self,
        msg: IncomingResponse,
    ) -> io::Result<()> {
        // propagate!(self, msg, on_response);

        Ok(())
    }

    pub fn get_name(&self) -> &String {
        &self.0.name
    }
}
