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

use crate::headers::SipHeaderParser;
use crate::macros::{b_map, read_while};
use crate::parser::{self, ALPHA_NUM, TOKEN};
use crate::util::is_valid_port;
use crate::{
    bytes::Bytes,
    macros::{read_until_byte, sip_parse_error, space},
    message::Transport,
    parser::Result,
    uri::{HostPort, Params},
};
use std::str;

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

#[derive(Debug, PartialEq, Eq, Default)]
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

// SIP Via Header
#[derive(Debug, PartialEq, Eq)]
pub struct Via<'a> {
    pub(crate) transport: Transport,
    pub(crate) sent_by: HostPort<'a>,
    pub(crate) params: Option<ViaParams<'a>>,
    pub(crate) comment: Option<&'a str>,
    pub(crate) others_params: Option<Params<'a>>,
}

impl<'a> Via<'a> {
    pub(crate) fn parse_params(
        bytes: &mut Bytes<'a>,
    ) -> Result<(Option<ViaParams<'a>>, Option<Params<'a>>)> {
        space!(bytes);
        if bytes.peek() != Some(&b';') {
            return Ok((None, None));
        }
        let mut params = ViaParams::default();
        let mut others = Params::new();
        while let Some(&b';') = bytes.peek() {
            bytes.next();
            let name = read_while!(bytes, is_via_param);
            let name = unsafe { str::from_utf8_unchecked(name) };
            let mut value = "";
            if let Some(&b'=') = bytes.peek() {
                bytes.next();
                let v = read_while!(bytes, is_via_param);
                value = unsafe { str::from_utf8_unchecked(v) };
            }
            match name {
                BRANCH_PARAM => params.set_branch(value),
                TTL_PARAM => params.set_ttl(value),
                MADDR_PARAM => params.set_maddr(value),
                RECEIVED_PARAM => params.set_received(value),
                RPORT_PARAM => {
                    if !value.is_empty() {
                        match value.parse::<u16>() {
                            Ok(port) if is_valid_port(port) => {
                                params.set_rport(port)
                            }
                            Ok(_) | Err(_) => {
                                return sip_parse_error!(
                                    "Via param rport is invalid!"
                                )
                            }
                        }
                    }
                }
                _ => {
                    others.set(name, Some(value));
                }
            }
            space!(bytes);
        }

        let others = if others.is_empty() {
            None
        } else {
            Some(others)
        };

        Ok((Some(params), others))
    }
}

impl<'a> SipHeaderParser<'a> for Via<'a> {
    const NAME: &'static [u8] = b"Via";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"v");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        //@TODO: handle LWS
        parser::parse_sip_v2(bytes)?;

        if bytes.next() != Some(&b'/') {
            return sip_parse_error!("Invalid via Hdr!");
        }
        let b = read_until_byte!(bytes, &b' ');
        let transport = Transport::from(b);

        space!(bytes);

        let sent_by = HostPort::parse(bytes)?;
        let (params, others_params) = Self::parse_params(bytes)?;

        let comment = if bytes.peek() == Some(&b'(') {
            bytes.next();
            let comment = read_until_byte!(bytes, &b')');
            bytes.next();
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

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"SIP/2.0/UDP bobspc.biloxi.com:5060;received=192.0.2.4\r\n";
        let mut bytes = Bytes::new(src);
        let via = Via::parse(&mut bytes);
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
        let mut bytes = Bytes::new(src);
        let via = Via::parse(&mut bytes);
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
