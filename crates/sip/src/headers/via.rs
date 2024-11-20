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

use scanner::util::is_valid_port;
use scanner::{space, until_byte, Scanner};

use crate::headers::{parse_param_sip, SipHeader};
use crate::macros::{b_map, parse_param};
use crate::parser::{SipParser, ALPHA_NUM, TOKEN};
use crate::{
    macros::sip_parse_error,
    message::Transport,
    parser::Result,
    uri::{HostPort, Params},
};
use std::str;

use super::Param;

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
fn parse_via_param<'a>(scanner: &mut Scanner<'a>) -> Result<Param<'a>> {
    unsafe { parse_param_sip(scanner, is_via_param) }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct ViaParams<'a> {
    ttl: Option<&'a str>,
    maddr: Option<&'a str>,
    received: Option<&'a str>,
    branch: Option<&'a str>,
    rport: Option<u16>,
}

impl<'a> ViaParams<'a> {
    pub fn set_branch(&mut self, branch: &'a str) {
        self.branch = Some(branch);
    }

    pub fn set_ttl(&mut self, ttl: &'a str) {
        self.ttl = Some(ttl);
    }
    pub fn set_maddr(&mut self, maddr: &'a str) {
        self.maddr = Some(maddr);
    }
    pub fn set_received(&mut self, received: &'a str) {
        self.received = Some(received);
    }
    pub fn set_rport(&mut self, rport: u16) {
        self.rport = Some(rport);
    }

    pub fn branch(&self) -> Option<&'a str> {
        self.branch
    }
}

/// The `Via` SIP header.
///
/// Indicates the path taken by the request so far and the
/// path that should be followed in routing responses.
#[derive(Debug, PartialEq, Eq)]
pub struct Via<'a> {
    pub(crate) transport: Transport,
    pub(crate) sent_by: HostPort<'a>,
    pub(crate) params: Option<ViaParams<'a>>,
    pub(crate) comment: Option<&'a str>,
    pub(crate) others_params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for Via<'a> {
    const NAME: &'static str = "Via";
    const SHORT_NAME: Option<&'static str> = Some("v");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        //@TODO: handle LWS
        SipParser::parse_sip_v2(scanner)?;

        if scanner.next() != Some(&b'/') {
            return sip_parse_error!("Invalid via Hdr!");
        }
        let b = until_byte!(scanner, &b' ');
        let transport = Transport::from(b);

        space!(scanner);

        let sent_by = HostPort::parse(scanner)?;
        let (params, others_params) = Self::parse_params(scanner)?;

        let comment = if scanner.peek() == Some(&b'(') {
            scanner.next();
            let comment = until_byte!(scanner, &b')');
            scanner.next();
            Some(str::from_utf8(comment)?)
        } else {
            None
        };

        Ok(Via {
            transport,
            sent_by,
            params,
            others_params,
            comment,
        })
    }
}

impl<'a> Via<'a> {
    pub(crate) fn parse_params(
        scanner: &mut Scanner<'a>,
    ) -> Result<(Option<ViaParams<'a>>, Option<Params<'a>>)> {
        space!(scanner);
        if scanner.peek() != Some(&b';') {
            return Ok((None, None));
        }
        let mut params = ViaParams::default();
        let mut rport_p = None;
        let others = parse_param!(
            scanner,
            parse_via_param,
            BRANCH_PARAM = params.branch,
            TTL_PARAM = params.ttl,
            MADDR_PARAM = params.maddr,
            RECEIVED_PARAM = params.received,
            RPORT_PARAM = rport_p
        );

        if let Some(rport) = rport_p
            .filter(|rport| !rport.is_empty())
            .and_then(|rpot| rpot.parse().ok())
        {
            if is_valid_port(rport) {
                params.set_rport(rport);
            } else {
                return sip_parse_error!("Via param rport is invalid!");
            }
        }

        Ok((Some(params), others))
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"SIP/2.0/UDP bobspc.biloxi.com:5060;received=192.0.2.4\r\n";
        let mut scanner = Scanner::new(src);
        let via = Via::parse(&mut scanner);
        let via = via.unwrap();

        assert_eq!(via.transport, Transport::UDP);
        assert_eq!(
            via.sent_by,
            HostPort::DomainName {
                host: "bobspc.biloxi.com",
                port: Some(5060)
            }
        );
        let params = via.params.unwrap();
        assert_eq!(params.received, Some("192.0.2.4"));

        let src = b"SIP/2.0/UDP 192.0.2.1:5060 ;received=192.0.2.207 \
        ;branch=z9hG4bK77asjd\r\n";
        let mut scanner = Scanner::new(src);
        let via = Via::parse(&mut scanner);
        let via = via.unwrap();

        assert_eq!(via.transport, Transport::UDP);
        assert_eq!(
            via.sent_by,
            HostPort::IpAddr {
                host: IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
                port: Some(5060)
            }
        );
        let params = via.params.unwrap();
        assert_eq!(params.received, Some("192.0.2.207"));
        assert_eq!(params.branch, Some("z9hG4bK77asjd"));
    }
}
