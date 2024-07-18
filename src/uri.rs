use std::{
    net::IpAddr,
    str::{self, FromStr},
};

use crate::{
    macros::{digits, peek},
    parser::reader::InputReader,
    util::{alphanum, escaped, uneserved},
};
use crate::{
    macros::{next, sip_parse_error},
    parser::SipParserError,
};

#[inline(always)]
fn password(byte: u8) -> bool {
    uneserved(byte) || pass(byte) || escaped(byte)
}

#[inline(always)]
fn pass(byte: u8) -> bool {
    match byte {
        b'&' | b'=' | b'+' | b'$' | b',' => true,
        _ => false,
    }
}

#[inline(always)]
fn user_unreserved(byte: u8) -> bool {
    match byte {
        b'&' | b'=' | b'+' | b'$' | b',' | b';' | b'?' | b'/' => true,
        _ => false,
    }
}

#[inline(always)]
fn user(byte: u8) -> bool {
    uneserved(byte) || user_unreserved(byte) || escaped(byte)
}

/*
Request-URI: The Request-URI is a SIP or SIPS URI as described in
           Section 19.1 or a general URI (RFC 2396 [5]).  It indicates
           the user or service to which this request is being addressed.
           The Request-URI MUST NOT contain unescaped spaces or control
           characters and MUST NOT be enclosed in "<>".
*/
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) name: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    pub fn parse(reader: &'a InputReader) -> Result<UserInfo<'a>, SipParserError> {
        let bytes = reader.read_while(user)?;
        let name = str::from_utf8(bytes)?;

        if peek!(reader) == Some(b':') {
            next!(reader);
            let bytes = reader.read_while(password)?;
            next!(reader);
            let pass = str::from_utf8(bytes)?;

            Ok(UserInfo {
                name,
                password: Some(pass),
            })
        } else {
            next!(reader);
            Ok(UserInfo {
                name,
                password: None,
            })
        }
    }
}

fn host(byte: u8) -> bool {
    alphanum(byte) || byte == b'_' || byte == b'-' || byte == b'.'
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Host<'a> {
    DomainName(&'a str),
    IpAddr(IpAddr),
}

impl<'a> Host<'a> {
    pub fn parse(reader: &'a InputReader) -> Result<Host<'a>, SipParserError> {
        let host = reader.read_while(host)?;
        let host = str::from_utf8(host)?;
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(Host::IpAddr(addr))
        } else {
            Ok(Host::DomainName(host))
        }
    }

    pub fn parse_ipv6(reader: &InputReader) -> Result<Host<'a>, SipParserError> {
        let host = reader.read_until_b(b']')?;
        let host = str::from_utf8(host)?;
        if let Ok(host) = host.parse() {
            next!(reader);
            Ok(Host::IpAddr(IpAddr::V6(host)))
        } else {
            sip_parse_error!("Error parsing Ipv6 Host!")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

// scheme
// user optional
// password optional
// str host required
// u32 port optional

// transport, maddr, ttl,user, method and lr
// str use_param  optional
// str method_param optional
// str transport_param  optional
// int ttl_param optional
// int lr_param optional
// str maddr_param optional

// struct sip_param/other_param other parameters group together
// struct sip_param/header_param optional
// SIP URI: sip:user:password@host:port;uri-parameters?headers
// SIPS URI: sips:user:password@host:port;uri-parameters?headers
#[derive(Debug, PartialEq, Eq)]
pub struct Uri<'a> {
    pub(crate) scheme: Scheme,
    pub(crate) user: Option<UserInfo<'a>>,
    pub(crate) host: Host<'a>,
    pub(crate) port: Option<u16>,
}

impl<'a> Uri<'a> {
    pub fn new(
        scheme: Scheme,
        user: Option<UserInfo<'a>>,
        host: Host<'a>,
        port: Option<u16>,
    ) -> Self {
        Uri {
            scheme,
            user,
            host,
            port,
        }
    }

    pub fn parse_port(reader: &InputReader) -> Result<Option<u16>, SipParserError> {
        let digits = digits!(reader);
        let digits = std::str::from_utf8(digits)?;

        match u16::from_str_radix(digits, 10) {
            Ok(port) => Ok(Some(port)),
            Err(_) => sip_parse_error!("Port is invalid integer!"),
        }
    }
}

//SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
pub struct NameAddr<'a> {
    display: Option<&'a str>,
    uri: Uri<'a>,
}
