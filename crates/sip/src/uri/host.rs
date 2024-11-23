use std::net::Ipv4Addr;
use std::str;
use std::{net::IpAddr, str::FromStr};

use reader::util::is_valid_port;
use reader::{until_byte, Reader};

use crate::{macros::sip_parse_error, parser::Result};

use super::is_host;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HostPort<'a> {
    DomainName { host: &'a str, port: Option<u16> },
    IpAddr { host: IpAddr, port: Option<u16> },
}

impl Default for HostPort<'_> {
    fn default() -> Self {
        Self::IpAddr {
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: Some(5060),
        }
    }
}

impl<'a> HostPort<'a> {
    fn with_addr(host: IpAddr, port: Option<u16>) -> Self {
        Self::IpAddr { host, port }
    }

    fn with_domain(host: &'a str, port: Option<u16>) -> Self {
        Self::DomainName { host, port }
    }

    pub fn host_as_string(&self) -> String {
        match self {
            HostPort::DomainName { host, .. } => host.to_string(),
            HostPort::IpAddr { host, .. } => host.to_string(),
        }
    }

    fn parse_port(reader: &mut Reader) -> Result<Option<u16>> {
        let Some(&b':') = reader.peek() else {
            return Ok(None);
        };
        reader.next();
        let digits = reader.read_num()?;
        if is_valid_port(digits) {
            Ok(Some(digits))
        } else {
            sip_parse_error!("Sip Uri Port is invalid!")
        }
    }

    fn parse_ipv6(reader: &mut Reader<'a>) -> Result<HostPort<'a>> {
        reader.must_read(b'[')?;
        // the '[' and ']' characters are removed from the host
        let host = until_byte!(reader, &b']');
        let host = str::from_utf8(host)?;
        reader.must_read(b']')?;

        match host.parse() {
            Ok(host) => Ok(Self::with_addr(host, Self::parse_port(reader)?)),
            Err(_) => sip_parse_error!("Error parsing Ipv6 HostPort!"),
        }
    }

    pub(crate) fn parse(reader: &mut Reader<'a>) -> Result<HostPort<'a>> {
        if let Some(&b'[') = reader.peek() {
            return Self::parse_ipv6(reader);
        }

        let host = unsafe { reader.read_as_str(is_host) };
        match IpAddr::from_str(host) {
            Ok(addr) => Ok(Self::with_addr(addr, Self::parse_port(reader)?)),
            Err(_) => Ok(Self::with_domain(host, Self::parse_port(reader)?)),
        }
    }
}
