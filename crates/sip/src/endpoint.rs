use tokio::sync::oneshot::{self};

use crate::message::{SipMethod, StatusLine};
use crate::service::Request;
use crate::transaction::{TsxKey, TsxSender};
use crate::transport::{
    IncomingInfo, IncomingRequest, IncomingResponse, OutgoingInfo,
    OutgoingResponse, Packet, Transport, TransportLayer, CRLF, END,
};
use crate::{
    headers::{Headers, Via},
    message::{
        HostPort, SipMessage, SipResponse, SipUri, StatusCode, UriBuilder,
    },
    parser::parse_sip_msg,
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transaction::TransactionLayer,
};
use std::{io, net::SocketAddr, ops::Deref, sync::Arc};

type TransportMessage = (Transport, Packet);

pub struct EndpointBuilder {
    name: String,
    dns_resolver: Resolver,
    tp_layer: TransportLayer,
    tsx_layer: TransactionLayer,
    // Accept, Allow, Supported
    capabilities: Headers,
    services: Vec<Box<dyn SipService>>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder {
            tp_layer: TransportLayer::new(),
            name: String::new(),
            capabilities: Headers::new(),
            dns_resolver: Resolver::default(),
            services: vec![],
            tsx_layer: TransactionLayer::default(),
        }
    }

    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.tp_layer.add(transport);

        self
    }

    pub fn with_service(mut self, service: impl SipService) -> Self {
        self.services.push(Box::new(service));

        self
    }

    pub fn build(self) -> Endpoint {
        Endpoint(Arc::new(Inner {
            tsx_layer: self.tsx_layer,
            tp_layer: self.tp_layer,
            name: self.name,
            capabilities: self.capabilities,
            dns_resolver: self.dns_resolver,
            services: self.services,
        }))
    }
}
pub struct Inner {
    tp_layer: TransportLayer,
    tsx_layer: TransactionLayer,
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
    pub async fn run(&self) -> io::Result<()> {
        let mut rx = self.tp_layer.initialize();
        while let Some(msg) = rx.recv().await {
            let endpt = self.clone();
            tokio::spawn(async move { endpt.on_transport_message(msg).await });
        }
        Ok(())
    }

    async fn on_transport_message(
        self,
        msg: TransportMessage,
    ) -> io::Result<()> {
        let (transport, packet) = msg;
        let msg = match packet.payload() {
            CRLF => {
                transport.send(END, packet.addr()).await?;
                return Ok(());
            }
            END => {
                return Ok(());
            }
            bytes => match parse_sip_msg(bytes) {
                Ok(sip) => sip,
                Err(err) => {
                    println!("ERROR ON PARSE MSG: {:#?}", err);
                    return Err(io::Error::other(err.message));
                }
            },
        };
        let info = IncomingInfo::new(packet, transport);
        match msg {
            SipMessage::Request(msg) => {
                let Ok(req_headers) = (&msg.headers).try_into() else {
                    return Err(io::Error::other("Could not parse headers"));
                };
                let req = IncomingRequest::new(msg, info, Some(req_headers));
                let _ = self.receive_request(req).await;
            }
            SipMessage::Response(msg) => {
                let res = IncomingResponse::new(msg, info);
                let _ = self.receive_response(res).await;
            }
        }
        Ok(())
    }

    pub async fn new_response(
        &self,
        req: &mut IncomingRequest,
        st_line: StatusLine,
    ) -> io::Result<OutgoingResponse> {
        let mut hdrs = req.req_hdrs.take().unwrap();
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
            let temp_uri = UriBuilder::new()
                .host(HostPort::new(maddr.clone(), Some(port)))
                .transport_param(transport.protocol())
                .get();
            let temp_uri = SipUri::Uri(temp_uri);
            let addresses = self.dns_resolver.resolve(&temp_uri).await?;
            let ServerAddress { addr, protocol } = addresses[0];
            // Find transport
            //TODO: the transport tp_layer must create a transport if it cannot find
            let transport = self.tp_layer.find(addr, protocol).ok_or(
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

    fn spawn_tsx_drop_notifier(
        self,
        key: TsxKey,
        notifier: oneshot::Receiver<()>,
    ) {
        tokio::spawn(async move {
            let _ = notifier.await;
            self.tsx_layer.remove(&key)
        });
    }

    pub async fn create_uas_tsx(
        &self,
        key: &TsxKey,
        request: &mut IncomingRequest,
    ) -> io::Result<(TsxSender, oneshot::Receiver<()>)> {
        if request.is_method(&SipMethod::Ack)
            || request.is_method(&SipMethod::Cancel)
        {
            return Err(io::Error::other(
                "ACK and CANCEL cannot create a transaction",
            ));
        }

        self.tsx_layer.create_uas_tsx(key, self, request).await
    }

    pub async fn receive_request(self, msg: IncomingRequest) -> io::Result<()> {
        let key = TsxKey::try_from(&msg).unwrap();
        // Check transaction.
        let Some(mut msg) = self.tsx_layer.handle_request(&key, msg).await?
        else {
            return Ok(());
        };

        // Create the server transaction.
        let (tsx, drop_notifier) = self.create_uas_tsx(&key, &mut msg).await?;

        self.clone().spawn_tsx_drop_notifier(key, drop_notifier);

        let mut request = Request {
            endpoint: self.clone(),
            msg: msg.into(),
            tsx,
        };

        // new_incoming
        for service in self.services.iter() {
            service.on_request(&mut request).await?;

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
