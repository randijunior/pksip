use pksip_util::util::is_valid_port;

use crate::headers::SipHeaderParse;
use crate::macros::parse_param;
use crate::message::Host;
use crate::parser::{self, ParseCtx, SIPV2};
use crate::{
    error::Result,
    macros::parse_error,
    message::TransportKind,
    message::{HostPort, Params},
};
use core::fmt;
use std::net::IpAddr;
use std::str::{self};
use std::sync::Arc;

const MADDR_PARAM: &str = "maddr";
const BRANCH_PARAM: &str = "branch";
const TTL_PARAM: &str = "ttl";
const RPORT_PARAM: &str = "rport";
const RECEIVED_PARAM: &str = "received";

/// The `Via` SIP header.
///
/// Indicates the path taken by the request so far and the
/// path that should be followed in routing responses.
///
/// # Examples
/// ```
/// # use pksip::headers::Via;
/// # use std::str::FromStr;
///
/// let input = "Via: SIP/2.0/UDP server10.biloxi.com;branch=z9hG4bKnashds8";
///
/// let via = Via::new_udp(
///     "server10.biloxi.com".parse().unwrap(),
///     Some("z9hG4bKnashds8"),
/// );
///
/// assert_eq!(input, via.to_string());
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Via<'a> {
    transport: TransportKind,
    sent_by: HostPort,
    ttl: Option<&'a str>,
    maddr: Option<Host>,
    received: Option<IpAddr>,
    branch: Option<&'a str>,
    rport: Option<u16>,
    comment: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> Via<'a> {
    /// Creates a new `Via` header with UDP transport and optional branch.
    ///
    /// # Arguments
    /// * `sent_by` - The host and optional port to which responses should be sent.
    /// * `branch` - Optional branch parameter to identify the transaction.
    pub fn new_udp(sent_by: HostPort, branch: Option<&'a str>) -> Self {
        Self {
            transport: TransportKind::Udp,
            sent_by,
            ttl: None,
            maddr: None,
            received: None,
            branch,
            rport: None,
            comment: None,
            params: None,
        }
    }
    /// Set the `received` parameter.
    pub fn set_received(&mut self, received: IpAddr) {
        self.received = Some(received);
    }

    /// Returns the `received` parameter.
    pub fn received(&self) -> Option<IpAddr> {
        self.received
    }

    /// Returns the `transport`.
    pub fn transport(&self) -> TransportKind {
        self.transport
    }

    /// Returns the `rport`.
    pub fn rport(&self) -> Option<u16> {
        self.rport
    }

    /// Set the sent_by field.
    pub fn set_sent_by(&mut self, sent_by: HostPort) {
        self.sent_by = sent_by;
    }

    /// Returns the branch parameter.
    pub fn branch(&self) -> Option<&str> {
        self.branch
    }

    /// Returns the sent_by field.
    pub fn sent_by(&self) -> &HostPort {
        &self.sent_by
    }

    /// Returns the `maddr` parameter.
    pub fn maddr(&self) -> &Option<Host> {
        &self.maddr
    }
}

impl fmt::Display for Via<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}/{} {}", Via::NAME, SIPV2, self.transport, self.sent_by)?;

        if let Some(rport) = self.rport {
            write!(f, ";rport={}", rport)?;
        }
        if let Some(received) = &self.received {
            write!(f, ";received={received}")?;
        }
        if let Some(ttl) = &self.ttl {
            write!(f, ";ttl={ttl}")?;
        }
        if let Some(maddr) = &self.maddr {
            write!(f, ";maddr={maddr}")?;
        }
        if let Some(branch) = &self.branch {
            write!(f, ";branch={branch}")?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{params}")?;
        }
        if let Some(comment) = &self.comment {
            write!(f, " ({comment})")?;
        }

        Ok(())
    }
}

impl<'a> SipHeaderParse<'a> for Via<'a> {
    const NAME: &'static str = "Via";
    const SHORT_NAME: &'static str = "v";
    /*
     * Via               =  ( "Via" / "v" ) HCOLON via-parm *(COMMA via-parm)
     * via-parm          =  sent-protocol LWS sent-by *( SEMI via-params )
     * via-params        =  via-ttl / via-maddr
     *                      / via-received / via-branch
     *                      / via-extension
     * via-ttl           =  "ttl" EQUAL ttl
     * via-maddr         =  "maddr" EQUAL host
     * via-received      =  "received" EQUAL (IPv4address / IPv6address)
     * via-branch        =  "branch" EQUAL token
     * via-extension     =  generic-param
     * sent-protocol     =  protocol-name SLASH protocol-version
     *                      SLASH transport
     * protocol-name     =  "SIP" / token
     * protocol-version  =  token
     * transport         =  "UDP" / "TCP" / "TLS" / "SCTP"
     *                      / other-transport
     * sent-by           =  host [ COLON port ]
     * ttl               =  1*3DIGIT ; 0 to 255
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        //@TODO: handle LWS
        parser.parse_sip_v2()?;

        let b = parser.read_until_byte(b' ');
        let transport = b.into();

        parser.take_ws();

        let sent_by = parser.parse_host_port()?;
        let mut branch = None;
        let mut ttl = None;
        let mut maddr = None;
        let mut received = None;
        let mut rport_p = None;
        let params = parse_param!(
            parser,
            parser::parse_via_param,
            BRANCH_PARAM = branch,
            TTL_PARAM = ttl,
            MADDR_PARAM = maddr,
            RECEIVED_PARAM = received,
            RPORT_PARAM = rport_p
        );
        let received = received.and_then(|r| r.parse().ok());
        let maddr = maddr.map(|a| match a.parse() {
            Ok(addr) => Host::IpAddr(addr),
            Err(_) => Host::DomainName(a.into()),
        });

        let rport = if let Some(rport) = rport_p
            .filter(|rport| !rport.is_empty())
            .and_then(|rpot| rpot.parse().ok())
        {
            if is_valid_port(rport) {
                Some(rport)
            } else {
                return parse_error!("Via param rport is invalid!");
            }
        } else {
            None
        };

        let comment = if parser.peek() == Some(&b'(') {
            parser.advance();
            let comment = parser.read_until_byte(b')');
            parser.advance();
            Some(str::from_utf8(comment)?)
        } else {
            None
        };

        Ok(Via {
            transport,
            sent_by,
            params,
            comment,
            ttl,
            maddr,
            received,
            branch,
            rport,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::message::Host;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"SIP/2.0/UDP bobspc.biloxi.com:5060;received=192.0.2.4\r\n";
        let mut scanner = ParseCtx::new(src);
        let via = Via::parse(&mut scanner);
        let via = via.unwrap();

        assert_eq!(via.transport, TransportKind::Udp);
        assert_eq!(
            via.sent_by,
            HostPort {
                host: Host::DomainName("bobspc.biloxi.com".into()),
                port: Some(5060)
            }
        );

        assert_eq!(via.received, Some("192.0.2.4".parse().unwrap()));

        let src = b"SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 \
        ;branch=z9hG4bK77asjd\r\n";
        let mut scanner = ParseCtx::new(src);
        let via = Via::parse(&mut scanner);
        let via = via.unwrap();

        assert_eq!(via.transport, TransportKind::Udp);
        assert_eq!(
            via.sent_by,
            HostPort {
                host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))),
                port: Some(5060)
            }
        );

        assert_eq!(via.received, Some("192.0.2.207".parse().unwrap()));
        assert_eq!(via.branch, Some("z9hG4bK77asjd".into()));
    }
}
