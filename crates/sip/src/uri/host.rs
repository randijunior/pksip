use std::net::Ipv4Addr;
use std::str;
use std::{net::IpAddr, str::FromStr};

use scanner::util::is_valid_port;
use scanner::{until_byte, Scanner};

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
    fn with_addr(addr: IpAddr, scanner: &mut Scanner<'a>) -> Result<Self> {
        Ok(Self::IpAddr {
            host: addr,
            port: Self::parse_port(scanner)?,
        })
    }

    fn with_domain(domain: &'a str, scanner: &mut Scanner<'a>) -> Result<Self> {
        Ok(Self::DomainName {
            host: domain,
            port: Self::parse_port(scanner)?,
        })
    }

    fn parse_port(scanner: &mut Scanner) -> Result<Option<u16>> {
        let Some(&b':') = scanner.peek() else {
            return Ok(None);
        };
        scanner.next();
        let digits = scanner.read_num()?;
        if is_valid_port(digits) {
            Ok(Some(digits))
        } else {
            sip_parse_error!("Sip Uri Port is invalid!")
        }
    }

    fn parse_ipv6(scanner: &mut Scanner<'a>) -> Result<HostPort<'a>> {
        scanner.must_read(b'[')?;
        // the '[' and ']' characters are removed from the host
        let host = until_byte!(scanner, &b']');
        let host = str::from_utf8(host)?;
        scanner.must_read(b']')?;

        match host.parse() {
            Ok(host) => Self::with_addr(host, scanner),
            Err(_) => sip_parse_error!("Error parsing Ipv6 HostPort!"),
        }
    }

    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<HostPort<'a>> {
        if let Some(&b'[') = scanner.peek() {
            return Self::parse_ipv6(scanner);
        }

        let host = unsafe { scanner.read_and_convert_to_str(is_host) };
        match IpAddr::from_str(host) {
            Ok(addr) => Self::with_addr(addr, scanner),
            Err(_) => Self::with_domain(host, scanner),
        }
    }
}
