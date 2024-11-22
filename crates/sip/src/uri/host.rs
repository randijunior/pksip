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
            port: None,
        }
    }
}

impl<'a> HostPort<'a> {
    fn with_addr(addr: IpAddr, reader: &mut Reader<'a>) -> Result<Self> {
        Ok(Self::IpAddr {
            host: addr,
            port: Self::parse_port(reader)?,
        })
    }

    fn with_domain(domain: &'a str, reader: &mut Reader<'a>) -> Result<Self> {
        Ok(Self::DomainName {
            host: domain,
            port: Self::parse_port(reader)?,
        })
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
            Ok(host) => Self::with_addr(host, reader),
            Err(_) => sip_parse_error!("Error parsing Ipv6 HostPort!"),
        }
    }

    pub(crate) fn parse(reader: &mut Reader<'a>) -> Result<HostPort<'a>> {
        if let Some(&b'[') = reader.peek() {
            return Self::parse_ipv6(reader);
        }

        let host = unsafe { reader.read_while_as_str(is_host) };
        match IpAddr::from_str(host) {
            Ok(addr) => Self::with_addr(addr, reader),
            Err(_) => Self::with_domain(host, reader),
        }
    }
}
