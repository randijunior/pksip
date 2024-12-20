use std::{
    io,
    net::{IpAddr, SocketAddr},
};

use crate::{
    msg::{Host, Scheme, TransportProtocol},
    transport::Transport,
};

pub struct ServerAddress {
    pub protocol: TransportProtocol,
    pub addr: SocketAddr,
}

pub struct Resolver {
    dns_resolver: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    pub async fn resolve(
        &self,
        host: &Host<'_>,
        port: Option<u16>,
        tp: Option<&Transport>,
        scheme: Scheme,
    ) -> io::Result<Vec<ServerAddress>> {
        // https://datatracker.ietf.org/doc/html/rfc3263#section-4.1
        // Arcording to RFC 3263, section 4.1:
        // If the URI specifies a transport protocol in the transport parameter,
        // that transport protocol SHOULD be used.
        // Otherwise, if no transport protocol is specified, but the TARGET is a
        //numeric IP address, the client SHOULD use UDP for a SIP URI, and TCP
        // for a SIPS URI.
        let is_ip_addr = host.is_ip_addr();

        let protocol = if let Some(tp) = tp {
            tp.get_protocol()
        } else {
            if is_ip_addr || port.is_some() {
                match scheme {
                    Scheme::Sip => TransportProtocol::UDP,
                    Scheme::Sips => TransportProtocol::TCP,
                }
            } else {
                //TODO: perform a NAPTR query for the domain in the URI
                TransportProtocol::UDP
            }
        };
        let port = protocol.get_port();
        let target = host.as_str();
        let result =
            self.dns_resolver.lookup_ip(target).await.map_err(|err| {
                io::Error::other(format!("Failed to lookup dns: {}", err))
            })?;

        let addresses = result
            .iter()
            .map(|addr| {
                let addr = SocketAddr::new(addr, port);
                ServerAddress { addr, protocol }
            })
            .collect();

        Ok(addresses)
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            dns_resolver:
                hickory_resolver::AsyncResolver::tokio_from_system_conf()
                    .expect("Failed to get DNS resolver"),
        }
    }
}
