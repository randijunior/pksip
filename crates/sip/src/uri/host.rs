use core::str;
use std::{net::IpAddr, str::FromStr};

use crate::{
    macros::{digits, read_until_byte, read_while, sip_parse_error},
    parser::SipParserError,
    scanner::Scanner,
    util::is_valid_port,
};

use super::is_host;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HostPort<'a> {
    DomainName { host: &'a str, port: Option<u16> },
    IpAddr { host: IpAddr, port: Option<u16> },
}

impl<'a> HostPort<'a> {
    fn parse_port(
        scanner: &mut Scanner,
    ) -> Result<Option<u16>, SipParserError> {
        if let Ok(Some(_)) = scanner.read_if(|b| b == &b':') {
            let digits = digits!(scanner);
            let digits = unsafe { str::from_utf8_unchecked(digits) };
            match digits.parse::<u16>() {
                Ok(port) if is_valid_port(port) => Ok(Some(port)),
                Ok(_) | Err(_) => {
                    sip_parse_error!("Sip Uri Port is invalid!")
                }
            }
        } else {
            Ok(None)
        }
    }
    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<HostPort<'a>, SipParserError> {
        if let Ok(Some(_)) = scanner.read_if(|b| b == &b'[') {
            // the '[' and ']' characters are removed from the host
            let host = read_until_byte!(scanner, &b']');
            let host = str::from_utf8(host)?;
            scanner.next();
            return if let Ok(host) = host.parse() {
                scanner.next();
                Ok(HostPort::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(scanner)?,
                })
            } else {
                sip_parse_error!("scannerError parsing Ipv6 HostPort!")
            };
        }
        let host = read_while!(scanner, is_host);
        let host = unsafe { str::from_utf8_unchecked(host) };
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(HostPort::IpAddr {
                host: addr,
                port: Self::parse_port(scanner)?,
            })
        } else {
            Ok(HostPort::DomainName {
                host,
                port: Self::parse_port(scanner)?,
            })
        }
    }
}
