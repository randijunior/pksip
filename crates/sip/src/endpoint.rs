use std::{io, sync::Arc};

use crate::{
    headers::Headers,
    resolver::Resolver,
    transport::{manager::TransportManager, IncomingMessage, Transport},
};

pub struct EndpointBuilder {
    manager: TransportManager,
    name: String,
    dns_resolver: Resolver,
    // Accept, Allow, Supported
    capabilities: Headers<'static>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder {
            manager: TransportManager::new(),
            name: String::new(),
            capabilities: Headers::new(),
            dns_resolver: Resolver::default(),
        }
    }

    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.manager.add(transport);

        self
    }

    pub fn build(self) -> Endpoint {
        let EndpointBuilder {
            manager,
            name,
            capabilities,
            dns_resolver,
        } = self;

        Endpoint(Arc::new(Inner {
            manager,
            name,
            capabilities,
            dns_resolver,
        }))
    }
}
pub struct Inner {
    manager: TransportManager,
    name: String,
    capabilities: Headers<'static>,
    dns_resolver: Resolver,
}

#[derive(Clone)]
pub struct Endpoint(Arc<Inner>);

impl Endpoint {
    pub async fn run(&self) -> io::Result<()> {
        self.0.manager.recv(self).await?;

        Ok(())
    }

    pub async fn endpt_recv_msg<'a>(&self, msg: IncomingMessage<'a>) {
        todo!()
    }

    pub fn get_name(&self) -> &String {
        &self.0.name
    }
}
