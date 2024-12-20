use std::{
    io,
    net::{IpAddr, SocketAddr},
    ops::Deref,
    sync::Arc,
};

use crate::{
    headers::{Header, Headers, Via},
    msg::{
        HostPort, SipResponse, SipUri, StatusCode, StatusLine,
        UriBuilder,
    },
    resolver::{Resolver, ServerAddress},
    service::SipService,
    transport::{
        manager::TransportManager, IncomingRequest, IncomingResponse,
        OutgoingInfo, Transport,
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

    async fn send(
        &self,
        msg: &'a IncomingRequest<'a>,
        st_line: StatusLine<'a>,
        headers: Option<Headers<'a>>,
        body: Option<&'a [u8]>,
    ) -> io::Result<()> {
        let hdrs = &msg.request().headers;
        let mut headers = headers.unwrap_or(Headers::default());

        // The parser check required headers
        let via_hdrs = hdrs.find_via();
        let callid = hdrs.find_callid().unwrap().clone();
        let from = hdrs.find_from().unwrap().clone();
        let mut to = hdrs.find_to().unwrap().clone();
        let cseq = hdrs.find_cseq().unwrap().clone();

        if to.tag.is_none() && st_line.code > StatusCode::Trying {
            to.tag = via_hdrs[0].branch;
        }
        let info = self
            .get_outgoing_info(
                via_hdrs[0],
                msg.transport(),
                msg.packet().addr()
            )
            .await?;

        let via_hdrs: Vec<Header> =
            via_hdrs.iter().map(|&v| Header::Via(v.clone())).collect();

        headers.append(&mut via_hdrs.into());
        headers.push(Header::CallId(callid));
        headers.push(Header::From(from));
        headers.push(Header::To(to));
        headers.push(Header::CSeq(cseq));

        let resp = SipResponse::new(st_line, headers, body);

        self.manager.send_response(info, resp).await
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
            let addresses = self
                .dns_resolver
                .resolve(&temp_uri)
                .await?;
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
mod tests {
    use async_trait::async_trait;

    use crate::{
        msg::SipMethod,
        transport::{udp::Udp, IncomingRequest},
    };

    use super::*;

    pub struct MyService;

    #[async_trait]
    impl SipService for MyService {
        fn name(&self) -> &str {
            "MyService"
        }
        async fn on_recv_req(
            &self,
            sip_server: &SipServer,
            inc: &mut Option<IncomingRequest>,
        ) {
            let msg = inc.take().unwrap();

            if msg.request().req_line.method != SipMethod::Ack {
                let _ = sip_server
                    .respond(
                        &msg,
                        StatusCode::NotImplemented.into(),
                        None,
                        None,
                    )
                    .await;
            }
        }
    }

    #[tokio::test]
    async fn test_req() {
        let sip_server = SipServerBuilder::new()
            .with_service(MyService)
            .with_transport(Udp::bind("127.0.0.1:5060").await.unwrap())
            .build();

        let _ = sip_server.run().await;
    }
}
