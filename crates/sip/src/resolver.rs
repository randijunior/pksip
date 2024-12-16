use crate::msg::{Scheme, SipUri, TransportProtocol};

pub struct ServerAddresses {}

pub struct Resolver {
    dns_resolver: hickory_resolver::TokioAsyncResolver,
}

impl Resolver {
    async fn resolve(&self, target: &SipUri<'_>) -> ServerAddresses {
        // https://datatracker.ietf.org/doc/html/rfc3263#section-4.1
        // Arcording to RFC 3263, section 4.1:
        // If the URI specifies a transport protocol in the transport parameter,
        // that transport protocol SHOULD be used.
        let transport = if let Some(transport_param) = target.transport_param()
        {
            transport_param
        } else {
            // Otherwise, if no transport protocol is specified, but the TARGET is a
            //numeric IP address, the client SHOULD use UDP for a SIP URI, and TCP
            // for a SIPS URI.
            let host_port = target.host_port();
            if let Some(_addr) = host_port.ip_addr() {
                match target.scheme() {
                    Scheme::Sip => TransportProtocol::UDP,
                    Scheme::Sips => TransportProtocol::TCP,
                }
            } else {
                todo!()
            }
        };

        todo!()
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
