#![deny(missing_docs)]
//! SIP Endpoint Builder
//!

use std::net::SocketAddr;
use std::sync::Arc;

use itertools::Itertools;

use crate::endpoint::resolver::Resolver;
use crate::endpoint::{Endpoint,Inner};
use crate::headers::{Header, Headers};
use crate::transaction::TransactionLayer;
use crate::transport::tcp::TcpStartup;
use crate::transport::udp::UdpStartup;
use crate::transport::{TransportLayer, TransportStartup};
use crate::{SipService};


/// Builder for creating a new SIP `Endpoint`.
pub struct Builder {
    name: String,
    resolver: Resolver,
    transport: TransportLayer,
    transaction: Option<TransactionLayer>,
    capabilities: Headers<'static>,
    services: Vec<Box<dyn SipService>>,
    transport_start: Vec<Box<dyn TransportStartup>>,
}

impl Builder {
    /// Creates a new default instance of `Builder` to construct a `Endpoint`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::Builder::new().with_name("My Endpoint").build();
    /// ```
    pub fn new() -> Self {
        Builder {
            transport: TransportLayer::new(),
            name: String::new(),
            capabilities: Headers::new(),
            resolver: Resolver::default(),
            services: vec![],
            transaction: None,
            transport_start: vec![],
        }
    }

    /// Sets the endpoint name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// let endpoint = endpoint::Builder::new().with_name("My Endpoint").build();
    /// ```
    pub fn with_name<T: AsRef<str>>(mut self, s: T) -> Self {
        self.name = s.as_ref().to_string();

        self
    }

    /// Add a new capability to the endpoint.
    pub fn add_capability(mut self, capability: Header<'static>) -> Self {
        self.capabilities.push(capability);

        self
    }

    /// Add a new builder for TCP transport on specified address.
    pub fn with_tcp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(TcpStartup::new(addr)));
        self
    }

    /// Add a new builder for TCP transport on specified address.
    pub fn with_udp(mut self, addr: SocketAddr) -> Self {
        self.transport_start.push(Box::new(UdpStartup::new(addr)));
        self
    }

    /// Adds a service to the endpoint.
    ///
    /// This function can be called multiple times to add additional services.
    /// If a service with the same name already exists, the new service will not
    /// be added.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl SipService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    /// let endpoint = endpoint::Builder::new().with_service(MyService).build();
    /// ```
    pub fn with_service(mut self, service: impl SipService) -> Self {
        if self.service_exists(service.name()) {
            return self;
        }
        self.services.push(Box::new(service));

        self
    }

    /// Add a collection of services to the endpoint.
    ///
    /// Similar to [`Builder::with_service`], but allows adding multiple
    /// services at once. Unlike `with_service`, this method expects the
    /// services to be passed as trait objects (`Box<dyn SipService>`)
    /// instead of concrete types.
    ///
    /// # Examples
    ///
    /// ```
    /// # use pksip::*;
    /// struct MyService;
    ///
    /// impl SipService for MyService {
    ///     fn name(&self) -> &str {
    ///         "MyService"
    ///     }
    /// }
    ///
    /// struct OtherService;
    ///
    /// impl SipService for OtherService {
    ///     fn name(&self) -> &str {
    ///         "OtherService"
    ///     }
    /// }
    ///
    /// let endpoint = endpoint::Builder::new()
    ///     .with_services([
    ///         Box::new(MyService) as Box<dyn SipService>,
    ///         Box::new(OtherService) as Box<dyn SipService>,
    ///     ])
    ///     .build();
    /// ```
    pub fn with_services<I>(mut self, services: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn SipService>>,
    {
        for service in services {
            if self.service_exists(service.name()) {
                continue;
            }
            self.services.push(service);
        }

        self
    }

    fn service_exists(&self, name: &str) -> bool {
        let exists = self.services.iter().any(|s| s.name() == name);
        if exists {
            log::warn!("Service with name '{}' already exists", name);
        }
        exists
    }

    /// Sets the transaction layer.
    pub fn with_transaction_layer(mut self, tsx_layer: TransactionLayer) -> Self {
        self.transaction = Some(tsx_layer);

        self
    }

    /// Finalize the builder into a `Endpoint`.
    pub async fn build(self) -> Endpoint {
        log::trace!("Creating endpoint...");
        log::debug!(
            "Services registered {}",
            format_args!("({})", self.services.iter().map(|s| s.name()).join(", "))
        );

        let endpoint = Endpoint(Arc::new(Inner {
            transaction: self.transaction,
            transport: self.transport,
            name: self.name,
            capabilities: self.capabilities,
            resolver: self.resolver,
            services: self.services.into_boxed_slice(),
        }));

        // let tx = endpoint.transport().sender();

        // for tp_start in self.transport_start {
        //     tp_start.start(tx.clone()).await.expect("Failed");
        // }

        endpoint
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}