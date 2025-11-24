use std::sync::Arc;

use itertools::Itertools;
use util::DnsResolver;

use super::{Endpoint, EndpointHandler};
use crate::{
    endpoint::{EndpointInner, EndpointRef}, headers::{Header, Headers}, transaction::manager::TransactionLayer, transport::TransportManager
};

/// EndpointBuilder for creating a new SIP `Endpoint`.
pub struct EndpointBuilder {
    name: String,
    resolver: DnsResolver,
    transaction: Option<TransactionLayer>,
    capabilities: Headers,
    handlers: Vec<Box<dyn EndpointHandler>>,
}

impl EndpointBuilder {
    /// Creates a new default instance of `EndpointBuilder` to
    /// construct a `Endpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My Endpoint")
    ///     .build();
    /// ```
    pub fn new() -> Self {
        EndpointBuilder {
            name: String::new(),
            capabilities: Headers::new(),
            resolver: DnsResolver::default(),
            handlers: Vec::new(),
            transaction: None,
        }
    }

    /// Sets the endpoint name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .with_name("My Endpoint")
    ///     .build();
    /// ```
    pub fn with_name<T: AsRef<str>>(mut self, s: T) -> Self {
        self.name = s.as_ref().to_string();

        self
    }

    /// Add a new capability to the endpoint.
    pub fn add_capability(mut self, capability: Header) -> Self {
        self.capabilities.push(capability);

        self
    }

    /// Adds a service to the endpoint.
    ///
    /// This function can be called multiple times to add
    /// additional handlers. If a service with the same
    /// name already exists, the new service will not be
    /// added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl EndpointHandler for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .add_service(MyService)
    ///     .build();
    /// ```
    pub fn add_service(mut self, service: impl EndpointHandler) -> Self {
        if self.service_exists(service.name()) {
            return self;
        }
        self.handlers.push(Box::new(service));

        self
    }

    /// Add a collection of handlers to the endpoint.
    ///
    /// Similar to [`EndpointBuilder::add_service`], but allows
    /// adding multiple handlers at once. Unlike
    /// `add_service`, this method expects the handlers
    /// to be passed as trait objects (`Box<dyn
    /// EndpointHandler>`) instead of concrete types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl EndpointHandler for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    ///
    /// struct OtherService;
    ///
    /// impl EndpointHandler for OtherService {
    ///     fn name(&self) -> &str {
    ///         "OtherService"
    ///     }
    /// }
    ///
    /// let endpoint = endpoint::EndpointBuilder::new()
    ///     .add_handlers([
    ///         Box::new(MyService) as Box<dyn EndpointHandler>,
    ///         Box::new(OtherService) as Box<dyn EndpointHandler>,
    ///     ])
    ///     .build();
    /// ```
    pub fn add_handlers<I>(mut self, handlers: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn EndpointHandler>>,
    {
        for service in handlers {
            if self.service_exists(service.name()) {
                continue;
            }
            self.handlers.push(service);
        }

        self
    }

    fn service_exists(&self, name: &str) -> bool {
        let exists = self.handlers.iter().any(|s| s.name() == name);
        if exists {
            log::warn!("Service with name '{}' already exists", name);
        }
        exists
    }

    /// Sets the transaction layer.
    pub fn add_transaction(mut self, tsx_layer: TransactionLayer) -> Self {
        self.transaction = Some(tsx_layer);

        self
    }

    /// Finalize the EndpointBuilder into a `Endpoint`.
    pub fn build(self) -> Endpoint {
        log::trace!("Creating endpoint...");
        log::debug!(
            "Handlers registered {}",
            format_args!("({})", self.handlers.iter().map(|s| s.name()).join(", "))
        );

        let endpoint = Endpoint {
            inner: EndpointRef(Arc::new(EndpointInner {
                transaction: self.transaction,
                transport: TransportManager::new(),
                name: self.name,
                capabilities: self.capabilities,
                resolver: self.resolver,
                handlers: self.handlers,
            })),
        };

        endpoint
    }
}

impl Default for EndpointBuilder {
    fn default() -> Self {
        Self::new()
    }
}
