use std::{io, net::SocketAddr};

use crate::message::{Host, Scheme, SipUri, TransportProtocol};

pub struct ServerAddress {
    pub protocol: TransportProtocol,
    pub addr: SocketAddr,
}

pub struct HostPortInfo<'a> {
    pub host: &'a Host,
    pub port: u16,
    pub protocol: TransportProtocol,
}

pub struct Resolver {
    dns_resolver: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    pub async fn resolve(
        &self,
        target: HostPortInfo<'_>,
    ) -> io::Result<Vec<ServerAddress>> {
        // https://datatracker.ietf.org/doc/html/rfc3263#section-4.1
        // Arcording to RFC 3263, section 4.1:
        // If the URI specifies a transport protocol in the transport parameter,
        // that transport protocol SHOULD be used.
        // Otherwise, if no transport protocol is specified, but the TARGET is a
        //numeric IP address, the client SHOULD use UDP for a SIP URI, and TCP
        // for a SIPS URI.
        let host_port = target.host;
        let protocol = if host_port.is_ip_addr() {
            TransportProtocol::UDP
        } else {
            //TODO: perform a NAPTR query for the domain in the URI
            TransportProtocol::UDP
        };
        let port = protocol.get_port();
        let result = match host_port {
            Host::DomainName(arc_str) => self
                .dns_resolver
                .lookup_ip(arc_str.as_ref() as &str)
                .await
                .map_err(|err| {
                    io::Error::other(format!(
                        "Failed to lookup dns: {}",
                        err
                    ))
                })?,
            Host::IpAddr(ip_addr) => {
                log::debug!("CAIU AQ!");
                let addr = SocketAddr::new(*ip_addr, port);
                return Ok(vec![ServerAddress { addr, protocol }]);
            }
        };

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
