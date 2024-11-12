use std::str;
use std::{net::IpAddr, str::FromStr};

use crate::{
    bytes::Bytes,
    macros::{sip_parse_error, until_byte},
    parser::Result,
    util::is_valid_port,
};

use super::is_host;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HostPort<'a> {
    DomainName { host: &'a str, port: Option<u16> },
    IpAddr { host: IpAddr, port: Option<u16> },
}

impl<'a> HostPort<'a> {
    fn with_addr(addr: IpAddr, bytes: &mut Bytes<'a>) -> Result<Self> {
        Ok(Self::IpAddr {
            host: addr,
            port: Self::parse_port(bytes)?,
        })
    }

    fn with_domain(domain: &'a str, bytes: &mut Bytes<'a>) -> Result<Self> {
        Ok(Self::DomainName {
            host: domain,
            port: Self::parse_port(bytes)?,
        })
    }

    fn parse_port(bytes: &mut Bytes) -> Result<Option<u16>> {
        let Some(&b':') = bytes.peek() else {
            return Ok(None);
        };
        bytes.next();
        let digits = bytes.parse_num()?;
        if is_valid_port(digits) {
            Ok(Some(digits))
        } else {
            sip_parse_error!("Sip Uri Port is invalid!")
        }
    }

    fn parse_ipv6(bytes: &mut Bytes<'a>) -> Result<HostPort<'a>> {
        bytes.must_read(b'[')?;
        // the '[' and ']' characters are removed from the host
        let host = until_byte!(bytes, &b']');
        let host = str::from_utf8(host)?;
        bytes.must_read(b']')?;

        match host.parse() {
            Ok(host) => Self::with_addr(host, bytes),
            Err(_) => sip_parse_error!("Error parsing Ipv6 HostPort!"),
        }
    }

    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<HostPort<'a>> {
        if let Some(&b'[') = bytes.peek() {
            return Self::parse_ipv6(bytes);
        }

        let host = unsafe { bytes.parse_str(is_host) };
        match IpAddr::from_str(host) {
            Ok(addr) => Self::with_addr(addr, bytes),
            Err(_) => Self::with_domain(host, bytes),
        }
    }
}
