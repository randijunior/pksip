use core::str;
use std::{net::IpAddr, str::FromStr};

use crate::{
    bytes::Bytes,
    macros::{digits, read_until_byte, read_while, sip_parse_error},
    parser::SipParserError,
    util::is_valid_port,
};

use super::is_host;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HostPort<'a> {
    DomainName { host: &'a str, port: Option<u16> },
    IpAddr { host: IpAddr, port: Option<u16> },
}

impl<'a> HostPort<'a> {
    fn parse_port(bytes: &mut Bytes) -> Result<Option<u16>, SipParserError> {
        if let Ok(Some(_)) = bytes.read_if(|b| b == &b':') {
            let digits = digits!(bytes);
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
        bytes: &mut Bytes<'a>,
    ) -> Result<HostPort<'a>, SipParserError> {
        if let Ok(Some(_)) = bytes.read_if(|b| b == &b'[') {
            // the '[' and ']' characters are removed from the host
            let host = read_until_byte!(bytes, &b']');
            let host = str::from_utf8(host)?;
            bytes.next();
            return if let Ok(host) = host.parse() {
                bytes.next();
                Ok(HostPort::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(bytes)?,
                })
            } else {
                sip_parse_error!("bytesError parsing Ipv6 HostPort!")
            };
        }
        let host = read_while!(bytes, is_host);
        let host = unsafe { str::from_utf8_unchecked(host) };
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(HostPort::IpAddr {
                host: addr,
                port: Self::parse_port(bytes)?,
            })
        } else {
            Ok(HostPort::DomainName {
                host,
                port: Self::parse_port(bytes)?,
            })
        }
    }
}
