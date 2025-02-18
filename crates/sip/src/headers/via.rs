/*
Via               =  ( "Via" / "v" ) HCOLON via-parm *(COMMA via-parm)
via-parm          =  sent-protocol LWS sent-by *( SEMI via-params )
via-params        =  via-ttl / via-maddr
                     / via-received / via-branch
                     / via-extension
via-ttl           =  "ttl" EQUAL ttl
via-maddr         =  "maddr" EQUAL host
via-received      =  "received" EQUAL (IPv4address / IPv6address)
via-branch        =  "branch" EQUAL token
via-extension     =  generic-param
sent-protocol     =  protocol-name SLASH protocol-version
                     SLASH transport
protocol-name     =  "SIP" / token
protocol-version  =  token
transport         =  "UDP" / "TCP" / "TLS" / "SCTP"
                     / other-transport
sent-by           =  host [ COLON port ]
ttl               =  1*3DIGIT ; 0 to 255
*/

use reader::util::is_valid_port;
use reader::{space, until, Reader};

use crate::headers::SipHeader;
use crate::macros::{b_map, parse_param};
use crate::message::Host;
use crate::parser::{self, SipParserError, ALPHA_NUM, SIPV2, TOKEN};
use crate::{
    macros::sip_parse_error,
    message::TransportProtocol,
    message::{HostPort, Params},
    parser::Result,
};
use core::fmt;
use std::net::{IpAddr, SocketAddr};
use std::str::{self, FromStr};

use crate::internal::{ArcStr, Param};

b_map!(VIA_PARAM_SPEC_MAP => b"[:]", ALPHA_NUM, TOKEN);

const MADDR_PARAM: &str = "maddr";
const BRANCH_PARAM: &str = "branch";
const TTL_PARAM: &str = "ttl";
const RPORT_PARAM: &str = "rport";
const RECEIVED_PARAM: &str = "received";

#[inline(always)]
fn is_via_param(b: &u8) -> bool {
    VIA_PARAM_SPEC_MAP[*b as usize]
}

// Parses a via param.
fn parse_via_param<'a>(reader: &mut Reader<'a>) -> Result<Param> {
    unsafe { Param::parse_unchecked(reader, is_via_param) }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ViaParams {
    ttl: Option<ArcStr>,
    maddr: Option<ArcStr>,
    received: Option<ArcStr>,
    branch: Option<ArcStr>,
    rport: Option<u16>,
}

impl ViaParams {
    pub fn set_branch(&mut self, branch: &str) {
        self.branch = Some(branch.into());
    }

    pub fn set_ttl(&mut self, ttl: &str) {
        self.ttl = Some(ttl.into());
    }
    pub fn set_maddr(&mut self, maddr: &str) {
        self.maddr = Some(maddr.into());
    }
    pub fn set_received(&mut self, received: &str) {
        self.received = Some(received.into());
    }
    pub fn set_rport(&mut self, rport: u16) {
        self.rport = Some(rport);
    }

    pub fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }
}

/// The `Via` SIP header.
///
/// Indicates the path taken by the request so far and the
/// path that should be followed in routing responses.
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Via {
    pub transport: TransportProtocol,
    pub sent_by: HostPort,
    pub ttl: Option<ArcStr>,
    pub maddr: Option<Host>,
    pub received: Option<IpAddr>,
    pub branch: Option<ArcStr>,
    pub rport: Option<u16>,
    pub comment: Option<ArcStr>,
    pub params: Option<Params>,
}

impl FromStr for Via {
    type Err = SipParserError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(&mut Reader::new(s.as_bytes()))
    }
}

impl fmt::Display for Via {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{} {}", SIPV2, self.transport, self.sent_by)?;

        if let Some(rport) = self.rport {
            write!(f, ";rport={rport}")?;
        }
        if let Some(received) = self.received {
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

impl SipHeader<'_> for Via {
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
    fn parse(reader: &mut Reader) -> Result<Self> {
        //@TODO: handle LWS
        parser::parse_sip_v2(reader)?;

        if reader.next() != Some(&b'/') {
            return sip_parse_error!("Invalid via Hdr!");
        }
        let b = until!(reader, &b' ');
        let transport = b.into();

        space!(reader);

        let sent_by = parser::parse_host_port(reader)?;
        let mut branch = None;
        let mut ttl = None;
        let mut maddr = None;
        let mut received = None;
        let mut rport_p = None;
        let params = parse_param!(
            reader,
            parse_via_param,
            BRANCH_PARAM = branch,
            TTL_PARAM = ttl,
            MADDR_PARAM = maddr,
            RECEIVED_PARAM = received,
            RPORT_PARAM = rport_p
        );
        let received = received.and_then(|r| r.parse().ok());
        let maddr = maddr.and_then(|a| match a.parse() {
            Ok(addr) => Some(Host::IpAddr(addr)),
            Err(_) => Some(Host::DomainName(a)),
        });

        let rport = if let Some(rport) = rport_p
            .filter(|rport| !rport.is_empty())
            .and_then(|rpot| rpot.parse().ok())
        {
            if is_valid_port(rport) {
                Some(rport)
            } else {
                return sip_parse_error!("Via param rport is invalid!");
            }
        } else {
            None
        };

        let comment = if reader.peek() == Some(&b'(') {
            reader.next();
            let comment = until!(reader, &b')');
            reader.next();
            Some(str::from_utf8(comment)?)
        } else {
            None
        };

        Ok(Via {
            transport,
            sent_by,
            params,
            comment: comment.map(|s| s.into()),
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
        let mut reader = Reader::new(src);
        let via = Via::parse(&mut reader);
        let via = via.unwrap();

        assert_eq!(via.transport, TransportProtocol::UDP);
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
        let mut reader = Reader::new(src);
        let via = Via::parse(&mut reader);
        let via = via.unwrap();

        assert_eq!(via.transport, TransportProtocol::UDP);
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
