use std::{io, net::SocketAddr, ops::Deref, sync::Arc};

use crate::{
    headers::{Header, Headers, Via},
    msg::{HostPort, SipResponse, SipUri, StatusCode, StatusLine, UriBuilder},
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transaction::ServerTransaction,
    transport::{
        manager::TransportManager, IncomingRequest, IncomingResponse,
        OutGoingResponse, OutgoingInfo, Transport,
    },
};

pub struct SipServerBuilder {
    manager: TransportManager,
    name: String,
    dns_resolver: Resolver,
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
        } = self;

        SipServer(Arc::new(Inner {
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
        st_line: StatusLine<'a>,
        headers: Option<Headers<'a>>,
        body: Option<&'a [u8]>,
    ) -> io::Result<()> {
        self.send(msg, st_line, headers, body).await
    }

    pub fn respond_tsx(&self) -> io::Result<ServerTransaction> {
        todo!()
    }

    pub async fn new_response(
        &self,
        msg: &'a IncomingRequest<'a>,
        st_line: StatusLine<'a>,
        extra_headers: Option<Headers<'a>>,
        body: Option<&'a [u8]>,
    ) -> io::Result<OutGoingResponse<'a>> {
        let hdrs = &msg.request().headers;

        // The parser check required headers
        let vias = hdrs.find_via();
        let callid = hdrs.find_callid().unwrap().call_id().unwrap();
        let from = hdrs.find_from().unwrap().from().unwrap();
        let to = hdrs.find_to().unwrap().to().unwrap();
        let cseq = hdrs.find_cseq().unwrap().cseq().unwrap();
        let top_most_via = vias[0].via().unwrap();

        let info = self
            .get_outgoing_info(
                top_most_via,
                msg.transport(),
                msg.packet().addr(),
            )
            .await?;

        let mut headers = Headers::new();

        headers.push(Header::Via(top_most_via.clone()));
        headers.push(Header::From(from.clone()));

        let mut to = to.clone();
        if to.tag.is_none() && st_line.code > StatusCode::Trying {
            to.tag = top_most_via.branch;
        }
        headers.push(Header::To(to));
        headers.push(Header::CallId(callid.clone()));
        headers.push(Header::CSeq(cseq.clone()));

        if let Some(mut extra) = extra_headers {
            headers.append(&mut extra);
        }

        Ok(OutGoingResponse {
            info,
            msg: SipResponse::new(st_line, headers, body),
        })
    }

    async fn send(
        &self,
        msg: &IncomingRequest<'a>,
        st_line: StatusLine<'a>,
        extra_headers: Option<Headers<'a>>,
        body: Option<&'a [u8]>,
    ) -> io::Result<()> {
        let resp = self.new_response(msg, st_line, extra_headers, body).await?;

        self.manager.send_response(resp).await
    }

    // follow SIP RFC 3261 in section 18.2.2
    async fn get_outgoing_info(
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
        let mut msg = msg.into();
        let svcs = self.services.iter();

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
