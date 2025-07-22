use std::{io, net::IpAddr};

use hickory_resolver::{error::ResolveError, lookup_ip::LookupIp};
use crate::{Result};

/// Resolver
pub struct Resolver {
    dns_resolver: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    async fn lookup(&self, host: &str) -> std::result::Result<LookupIp, ResolveError> {
        self.dns_resolver.lookup_ip(host).await
    }
    /// Resolve a single.
    pub async fn resolve(&self, host: &str) -> Result<IpAddr> {
        Ok(self
            .lookup(host)
            .await
            .map_err(|err| io::Error::other(format!("Failed to lookup DNS: {}", err)))?
            .iter()
            .next()
            .unwrap())
    }
    /// Resolve a all.
    pub async fn resolve_all(&self, host: &str) -> Result<Vec<IpAddr>> {
        let result = self
            .lookup(host)
            .await
            .map_err(|err| io::Error::other(format!("Failed to lookup dns: {}", err)))?;

        let addresses = result.iter().collect();

        Ok(addresses)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            dns_resolver: hickory_resolver::AsyncResolver::tokio_from_system_conf()
                .expect("Failed to get DNS resolver"),
        }
    }
}