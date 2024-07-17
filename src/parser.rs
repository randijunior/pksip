use reader::{InputReader, ReaderError};

use crate::{
    macros::{alpha, digits, newline, next, peek, sip_parse_error, space},
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    uri::{Host, Scheme, Uri, UserInfo},
    util::is_newline,
};

use std::{
    net::{IpAddr, Ipv6Addr},
    str::{self, FromStr},
};

mod cursor;
mod reader;

const SIPV2: &[u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl<'a> From<ReaderError<'a>> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line: {}, column: {}, kind: {:?}, input: '{}'",
                err.pos.line,
                err.pos.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

#[inline(always)]
fn alpha(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}
#[inline(always)]
fn user_unreserved(byte: u8) -> bool {
    match byte {
        b'&' | b'=' | b'+' | b'$' | b',' | b';' | b'?' | b'/' => true,
        _ => false,
    }
}
#[inline(always)]
fn mark(byte: u8) -> bool {
    match byte {
        b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' => true,
        _ => false,
    }
}

#[inline(always)]
fn uneserved(byte: u8) -> bool {
    mark(byte) || alpha(byte)
}
#[inline(always)]
fn escaped(byte: u8) -> bool {
    byte == b'%'
}

#[inline(always)]
fn user(byte: u8) -> bool {
    uneserved(byte) || user_unreserved(byte) || escaped(byte)
}
#[inline(always)]
fn pass(byte: u8) -> bool {
    match byte {
        b'&' | b'=' | b'+' | b'$' | b',' => true,
        _ => false,
    }
}
#[inline(always)]
fn password(byte: u8) -> bool {
    uneserved(byte) || pass(byte) || escaped(byte)
}

fn host(byte: u8) -> bool {
    alpha(byte) || byte == b'_' || byte == b'-' || byte == b'.'
}

#[inline(always)]
fn maybe_has_user(reader: &InputReader) -> Option<u8> {
    for &byte in reader.as_slice().iter() {
        match byte {
            b'@' | b' ' | b'\n' | b'>' => return Some(byte),
            _ => continue,
        }
    }
    None
}

#[inline]
pub fn parse_status_line<'a>(
    reader: &'a InputReader,
) -> Result<StatusLine<'a>, SipParserError> {
    reader.tag(SIPV2)?;

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let status_code = SipStatusCode::from(digits);
    let bytes = reader.read_while(|b| !is_newline(b))?;

    if let Ok(rp) = str::from_utf8(bytes) {
        newline!(reader);
        Ok(StatusLine::new(status_code, rp))
    } else {
        sip_parse_error!("Reason phrase is invalid utf8!")
    }
}

fn parse_uri_host<'a>(reader: &'a InputReader) -> Result<Host<'a>, SipParserError> {
    if let Some(_) = reader.next_if(|b| b == b'[')? {
        match str::from_utf8(reader.read_until_b(b']')?) {
            Ok(host_str) => {
                if let Ok(host) = host_str.parse() {
                    next!(reader);
                    Ok(Host::IpAddr(IpAddr::V6(host)))
                } else {
                    sip_parse_error!("Error parsing Ipv6 Host!")
                }
            }
            Err(_) => sip_parse_error!("Sip Ipv6 host is invalid utf8!"),
        }
    } else {
        let host = reader.read_while(host)?;
        match str::from_utf8(host) {
            Ok(host) => {
                if let Ok(addr) = IpAddr::from_str(host) {
                    Ok(Host::IpAddr(addr))
                } else {
                    Ok(Host::DomainName(host))
                }
            }
            Err(_) => sip_parse_error!("Sip host is invalid utf8"),
        }
    }
}

fn parse_port(reader: &InputReader) -> Result<Option<u16>, SipParserError> {
    if let Some(_) = reader.next_if(|b| b == b':')? {
        let digits = digits!(reader);
        match std::str::from_utf8(digits) {
            Ok(digits) => match u16::from_str_radix(digits, 10) {
                Ok(port) => Ok(Some(port)),
                Err(_) => sip_parse_error!("Port is invalid!"),
            },
            Err(_) => sip_parse_error!("Port is invalid utf8"),
        }
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_user_and_pass<'a>(
    reader: &'a InputReader,
) -> Result<UserInfo<'a>, SipParserError> {
    let bytes = reader.read_while(user)?;
    match str::from_utf8(bytes) {
        Ok(user) => {
            if peek!(reader) == Some(b':') {
                next!(reader);
                let bytes = reader.read_while(password)?;
                next!(reader);

                match str::from_utf8(bytes) {
                    Ok(pass) => Ok(UserInfo::new(user, Some(pass))),
                    Err(_) => sip_parse_error!("Pass is invalid utf8!"),
                }
            } else {
                next!(reader);
                Ok(UserInfo::new(user, None))
            }
        }
        Err(_) => sip_parse_error!("User is invalid utf8!"),
    }
}

#[inline]
pub fn parse_request_line<'a>(
    reader: &'a InputReader,
) -> Result<RequestLine<'a>, SipParserError> {
    let method = SipMethod::from(alpha!(reader));

    space!(reader);

    let scheme = match reader.read_until_b(b':')? {
        b"sip" => Ok(Scheme::Sip),
        b"sips" => Ok(Scheme::Sips),
        _ => sip_parse_error!("Can't parse sip uri scheme"),
    }?;

    next!(reader);

    let has_user = maybe_has_user(reader).is_some_and(|b| b == b'@');
    let user_info = if has_user {
        Some(parse_user_and_pass(reader)?)
    } else {
        None
    };
    let host = parse_uri_host(reader)?;
    let port = parse_port(reader)?;
    let uri = Uri::new(scheme, user_info, host, port);

    newline!(reader);

    Ok(RequestLine { method, uri })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_line() {
        let sc_ok = SipStatusCode::Ok;
        let buf = "SIP/2.0 200 OK\r\n".as_bytes();
        let reader = InputReader::new(buf);

        assert_eq!(
            parse_status_line(&reader),
            Ok(StatusLine {
                status_code: sc_ok,
                reason_phrase: sc_ok.reason_phrase()
            })
        );
        let sc_not_found = SipStatusCode::NotFound;
        let buf = "SIP/2.0 404 Not Found\r\n".as_bytes();
        let reader = InputReader::new(buf);

        assert_eq!(
            parse_status_line(&reader),
            Ok(StatusLine {
                status_code: sc_not_found,
                reason_phrase: sc_not_found.reason_phrase()
            })
        );
    }

    #[test]
    fn benchmark_req_line() {
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let size = msg.len();
        let mut counter = 0;
        let now = std::time::Instant::now();
        loop {
            let reader = InputReader::new(msg);
            assert!(parse_request_line(&reader).is_ok(),);
            counter += 1;
            if now.elapsed().as_secs() == 1 {
                break;
            }
        }

        println!(
            "{} mbytes per second, count sip messages: {}",
            (size * counter) / 1024 / 1024,
            counter
        );
    }

    #[test]
    fn benchmark() {
        let sc_ok = SipStatusCode::Ok;
        let msg = "SIP/2.0 200 OK\r\n".as_bytes();
        let size = msg.len();
        let mut counter = 0;
        let now = std::time::Instant::now();
        loop {
            let reader = InputReader::new(msg);
            assert_eq!(
                parse_status_line(&reader),
                Ok(StatusLine {
                    status_code: sc_ok,
                    reason_phrase: sc_ok.reason_phrase()
                })
            );
            counter += 1;
            if now.elapsed().as_secs() == 1 {
                break;
            }
        }

        println!(
            "{} mbytes per second, count sip messages: {}",
            (size * counter) / 1024 / 1024,
            counter
        );
    }
}
