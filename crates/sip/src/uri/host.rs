use std::str;
use std::{net::IpAddr, str::FromStr};

use crate::{
    bytes::Bytes,
    macros::{until_byte, read_while, sip_parse_error},
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
    fn parse_port(bytes: &mut Bytes) -> Result<Option<u16>> {
        let Some(&b':') = bytes.peek() else {
            return Ok(None)
        };
        bytes.next();
        let digits = bytes.parse_num()?;
        if is_valid_port(digits) {
            Ok(Some(digits))
        } else {
            sip_parse_error!("Sip Uri Port is invalid!")
        }
    }
    pub(crate) fn parse(
        bytes: &mut Bytes<'a>,
    ) -> Result<HostPort<'a>> {
        if let Ok(Some(_)) = bytes.read_if(|b| b == &b'[') {
            // the '[' and ']' characters are removed from the host
            let host = until_byte!(bytes, &b']');
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
        let host = unsafe { bytes.parse_str(is_host) };
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
