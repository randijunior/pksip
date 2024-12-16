use std::{io, ops::Deref, sync::Arc};

use crate::{
    headers::Headers,
    resolver::Resolver,
    service::SipService,
    transport::{manager::TransportManager, IncomingMessage, Transport},
};

pub struct EndpointBuilder {
    manager: TransportManager,
    name: String,
    dns_resolver: Resolver,
    // Accept, Allow, Supported
    capabilities: Headers<'static>,
    services: Vec<Box<dyn SipService>>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder {
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

    pub fn build(self) -> Endpoint {
        let EndpointBuilder {
            manager,
            name,
            capabilities,
            dns_resolver,
            services,
        } = self;

        Endpoint(Arc::new(Inner {
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
pub struct Endpoint(Arc<Inner>);

impl Deref for Endpoint {
    type Target = Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Endpoint {
    pub async fn run(&self) -> io::Result<()> {
        self.manager.recv(self).await?;

        Ok(())
    }

    pub async fn endpt_recv_msg<'a>(&self, msg: IncomingMessage<'a>) {
        let mut msg = msg.into();
        for svc in self.services.iter() {
            svc.on_recv_req(self, &mut msg).await;

            if let None = msg {
                break;
            }
        }
    }

    pub fn get_name(&self) -> &String {
        &self.0.name
    }
}
