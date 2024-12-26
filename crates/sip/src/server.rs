use std::{io, net::SocketAddr, ops::Deref, sync::Arc};

use crate::{
    headers::{Headers, Via},
    msg::{HostPort, SipResponse, SipUri, StatusCode, UriBuilder},
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transaction::{ServerTransaction, Transactions, TsxKey},
    transport::{
        manager::TransportManager, IncomingRequest, IncomingResponse,
        OutGoingResponse, OutgoingInfo, RequestHeaders, Transport,
    },
};

pub struct SipServerBuilder {
    manager: TransportManager,
    name: String,
    dns_resolver: Resolver,
    transactions: Transactions<'static>,
    // Accept, Allow, Supported
    capabilities: Headers<'static>,
    services: Vec<Box<dyn SipService>>,
}

impl SipServerBuilder {
    pub fn new() -> Self {
        SipServerBuilder {
            manager: TransportManager::new(),
            name: String::new(),
            capabilities: Headers::new(),
            dns_resolver: Resolver::default(),
            services: vec![],
            transactions: Transactions::default(),
        }
    }

    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.manager.add(transport);

        self
    }

    pub fn with_service(mut self, service: impl SipService) -> Self {
        self.services.push(Box::new(service));

        self
    }

    pub fn build(self) -> SipServer {
        let SipServerBuilder {
            manager,
            name,
            capabilities,
            dns_resolver,
            services,
            transactions,
        } = self;

        SipServer(Arc::new(Inner {
            transactions,
            manager,
            name,
            capabilities,
            dns_resolver,
            services,
        }))
    }
}
pub struct Inner {
    manager: TransportManager,
    name: String,
    capabilities: Headers<'static>,
    dns_resolver: Resolver,
    services: Vec<Box<dyn SipService>>,
    //tsx_handlers
    transactions: Transactions<'static>,
}

#[derive(Clone)]
pub struct SipServer(Arc<Inner>);

impl Deref for SipServer {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> SipServer {
    pub async fn run(&self) -> io::Result<()> {
        self.manager.recv(self).await?;

        Ok(())
    }

    pub async fn respond(
        &self,
        msg: &IncomingRequest<'a>,
        res: SipResponse<'a>,
    ) -> io::Result<()> {
        self.send(msg, res).await
    }

    pub async fn respond_tsx(
        &self,
        msg: &'a IncomingRequest<'a>,
        res: SipResponse<'a>,
    ) -> io::Result<ServerTransaction> {
        let resp = self.new_response_from_request(msg, res).await?;

        todo!()
    }

    pub async fn new_response_from_request(
        &self,
        msg: &'a IncomingRequest<'a>,
        res: SipResponse<'a>,
    ) -> io::Result<OutGoingResponse<'a>> {
        let hdrs = &msg.request().headers;
        let mut req_hdrs: RequestHeaders<'a> = hdrs.into();
        let topmost_via = req_hdrs.via.first().unwrap();

        let info = self
            .get_outgoing_info(
                topmost_via,
                msg.transport(),
                msg.packet().addr(),
            )
            .await?;

        if req_hdrs.to.tag.is_none() && res.st_line.code > StatusCode::Trying {
            req_hdrs.to.tag = topmost_via.branch;
        }

        Ok(OutGoingResponse {
            req_hdrs,
            info,
            msg: res,
        })
    }

    async fn send(
        &self,
        msg: &IncomingRequest<'a>,
        res: SipResponse<'a>,
    ) -> io::Result<()> {
        let resp = self.new_response_from_request(msg, res).await?;

        self.manager.send_response(resp).await
    }

    // follow SIP RFC 3261 in section 18.2.2
    pub async fn get_outgoing_info(
        &self,
        via: &Via<'_>,
        tp: &Transport,
        src_addr: SocketAddr,
    ) -> io::Result<OutgoingInfo> {
        if tp.reliable() {
            // Tcp, TLS, etc..
            todo!()
        } else if let Some(maddr) = &via.maddr {
            // Otherwise, if the Via header field value contains a "maddr"
            // parameter, the response MUST be forwarded to the address listed
            // there, using the port indicated in "sent-by", or port 5060 if
            // none is present.
            let port = via.sent_by.port.unwrap_or(5060);
            // If the address is a multicast address, the response SHOULD be sent
            // using the TTL indicated in the "ttl" parameter, or with a TTL of 1
            // if that parameter is not present.
            let temp_uri = UriBuilder::new()
                .host(HostPort::new(maddr.clone(), Some(port)))
                .transport_param(tp.get_protocol())
                .get();
            let temp_uri = SipUri::Uri(temp_uri);
            let addresses = self.dns_resolver.resolve(&temp_uri).await?;
            let ServerAddress { addr, protocol } = addresses[0];
            // Find transport
            //TODO: the transport manager must create a transport if it cannot find
            let transport = self.manager.find(addr, protocol).ok_or(
                io::Error::other("Coun'd not find a suitable transport!"),
            )?;

            Ok(OutgoingInfo { addr, transport })
        } else if let Some(rport) = via.rport {
            // MUST use the "received" and "rport" parameter.
            let addr = via.received.unwrap_or(src_addr.ip());
            let addr = SocketAddr::new(addr, rport);
            Ok(OutgoingInfo {
                addr,
                transport: tp.clone(),
            })
        } else {
            Ok(OutgoingInfo {
                addr: src_addr,
                transport: tp.clone(),
            })
        }
    }

    pub async fn sip_server_recv_req(&self, msg: IncomingRequest<'a>) {
        let mut msg = msg.into();
        let svcs = self.services.iter();

        for svc in svcs {
            svc.on_recv_req(self, &mut msg).await;
            if msg.is_none() {
                break;
            }
        }
    }

    pub async fn sip_server_recv_res(&self, msg: IncomingResponse<'a>) {
        let svcs = self.services.iter();
        let key = TsxKey::from_res(&msg);

        if let Some(mut tsx) = self.transactions.find_tsx(key) {
            tsx.handle_response(&msg);
            let mut msg = msg.into();
            for svc in svcs {
                svc.on_tsx_res(self, &mut msg, &tsx).await;
                if msg.is_none() {
                    break;
                }
            }
            return;
        }


        let mut msg = msg.into();
        for svc in svcs {
            svc.on_recv_res(self, &mut msg).await;
            if msg.is_none() {
                break;
            }
        }
    }

    pub fn get_name(&self) -> &String {
        &self.0.name
    }
}

#[cfg(test)]
mod tests {}
