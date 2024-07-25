use crate::{
    byte_reader::{ByteReader, ByteReaderError},
    macros::{
        alpha, b_map, digits, newline, read_while, sip_parse_error, space, tag,
        until_byte, until_newline,
    },
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    uri::{GenericParams, Host, Scheme, Uri, UserInfo},
};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    str::{FromStr, Utf8Error},
};
const ALPHA_NUM: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const UNRESERVED: &[u8] = b"-_.!~*'()%";
const ESCAPED: &[u8] = b"%";
const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
const TOKEN: &[u8] = b"-.!%*_`'~+";
const PASS: &[u8] = b"&=+$,";
const HOST: &[u8] = b"_-.";

pub(crate) const USER_PARAM: &[u8] = "user".as_bytes();
pub(crate) const METHOD_PARAM: &[u8] = "method".as_bytes();
pub(crate) const TANSPORT_PARAM: &[u8] = "transport".as_bytes();
pub(crate) const TTL_PARAM: &[u8] = "ttl".as_bytes();
pub(crate) const LR_PARAM: &[u8] = "lr".as_bytes();
pub(crate) const MADDR_PARAM: &[u8] = "maddr".as_bytes();

// A-Z a-z 0-9 -_.!~*'() &=+$,;?/%
// For reading user part on sip uri.
b_map!(USER_SPEC_MAP => ALPHA_NUM, UNRESERVED, USER_UNRESERVED, ESCAPED);
// A-Z a-z 0-9 -_.!~*'() &=+$,%
// For reading password part on sip uri.
b_map!(PASS_SPEC_MAP => ALPHA_NUM, UNRESERVED, ESCAPED, PASS);
// A-Z a-z 0-9 -_.
b_map!(HOST_SPEC_MAP => ALPHA_NUM, HOST);
// "[]/:&+$"  "-_.!~*'()" "%"
b_map!(PARAM_SPEC_MAP => b"[]/:&+$", ALPHA_NUM, UNRESERVED, ESCAPED);
// "[]/?:+$"  "-_.!~*'()" "%"
b_map!(HDR_SPEC_MAP => b"[]/?:+$", ALPHA_NUM, UNRESERVED, ESCAPED);

#[inline]
fn is_user(b: u8) -> bool {
    USER_SPEC_MAP[b as usize]
}

#[inline]
fn is_pass(b: u8) -> bool {
    PASS_SPEC_MAP[b as usize]
}

pub struct SipParser<'a> {
    reader: ByteReader<'a>,
}

impl<'a> SipParser<'a> {
    pub fn new(i: &'a [u8]) -> Self {
        SipParser {
            reader: ByteReader::new(i),
        }
    }

    fn parse_scheme(reader: &mut ByteReader) -> Result<Scheme> {
        match until_byte!(reader, b':') {
            b"sip" => Ok(Scheme::Sip),
            b"sips" => Ok(Scheme::Sips),
            // Unsupported URI scheme
            _ => sip_parse_error!("Can't parse sip uri scheme"),
        }
    }

    fn has_user(reader: &ByteReader) -> bool {
        let mut matched = None;
        for &byte in reader.as_ref().iter() {
            match byte {
                b'@' | b' ' | b'\n' | b'>' => {
                    matched = Some(byte);
                    break;
                }
                _ => continue,
            }
        }
        matched == Some(b'@')
    }

    fn parse_user(reader: &mut ByteReader<'a>) -> Result<Option<UserInfo<'a>>> {
        if Self::has_user(reader) {
            let bytes = read_while!(reader, is_user);
            let name = str::from_utf8(bytes)?;

            if reader.peek() == Some(b':') {
                reader.next();
                let bytes = read_while!(reader, is_pass);
                reader.next();

                Ok(Some(UserInfo {
                    name,
                    password: Some(str::from_utf8(bytes)?),
                }))
            } else {
                reader.next();
                Ok(Some(UserInfo {
                    name,
                    password: None,
                }))
            }
        } else {
            Ok(None)
        }
    }

    fn parse_host(reader: &mut ByteReader<'a>) -> Result<Host<'a>> {
        if let Some(_) = reader.read_if(|b| b == b'[') {
            // the '[' and ']' characters are removed from the host
            reader.next();
            let host = until_byte!(reader, b']');
            reader.next();
            let host = str::from_utf8(host)?;
            if let Ok(host) = host.parse() {
                reader.next();
                Ok(Host::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(reader)?,
                })
            } else {
                sip_parse_error!("Error parsing Ipv6 Host!")
            }
        } else {
            let host = read_while!(reader, |b| HOST_SPEC_MAP[b as usize]);
            let host = str::from_utf8(host)?;
            if let Ok(addr) = IpAddr::from_str(host) {
                Ok(Host::IpAddr {
                    host: addr,
                    port: Self::parse_port(reader)?,
                })
            } else {
                Ok(Host::DomainName {
                    host,
                    port: Self::parse_port(reader)?,
                })
            }
        }
    }

    fn parse_port(reader: &mut ByteReader) -> Result<Option<u16>> {
        if let Some(_) = reader.read_if(|b| b == b':') {
            let digits = digits!(reader);
            let digits = str::from_utf8(digits)?;

            match u16::from_str_radix(digits, 10) {
                Ok(port) => Ok(Some(port)),
                Err(_) => sip_parse_error!("Port is invalid integer!"),
            }
        } else {
            Ok(None)
        }
    }

    fn parse_sip_uri(reader: &mut ByteReader<'a>) -> Result<Uri<'a>> {
        let scheme = Self::parse_scheme(reader)?;
        // take ':'
        reader.next();

        let user = Self::parse_user(reader)?;
        let host = Self::parse_host(reader)?;
        let mut user_param = None;
        let mut method_param = None;
        let mut transport_param = None;
        let mut ttl_param = None;
        let mut lr_param = None;
        let mut maddr_param = None;
        let mut other_params = None;
        if reader.peek() == Some(b';') {
            let mut others = HashMap::new();
            loop {
                if reader.peek() != Some(b';') {
                    break;
                }
                reader.next();
                let name = read_while!(reader, |b| PARAM_SPEC_MAP[b as usize]);
                let value = if reader.peek() == Some(b'=') {
                    reader.next();
                    let value = read_while!(reader, |b| PARAM_SPEC_MAP[b as usize]);
                    str::from_utf8(value)?
                } else {
                    ""
                };
                match name {
                    USER_PARAM => user_param = Some(value),
                    METHOD_PARAM => method_param = Some(value),
                    TANSPORT_PARAM => transport_param = Some(value),
                    TTL_PARAM => ttl_param = Some(value),
                    LR_PARAM => lr_param = Some(value),
                    MADDR_PARAM => maddr_param = Some(value),
                    _ => {
                        others.insert(str::from_utf8(name)?, value);
                    }
                }
            }
            if others.len() > 0 {
                other_params = Some(GenericParams { params: others });
            }
        }
        let mut header_params = None;
        if reader.peek() == Some(b'?') {
            let mut params = HashMap::new();
            loop {
                // take '?' or '&'
                reader.next();
                let name = read_while!(reader, |b| HDR_SPEC_MAP[b as usize]);
                let name = str::from_utf8(name)?;
                let value = if reader.peek() == Some(b'=') {
                    reader.next();
                    let value = read_while!(reader, |b| HDR_SPEC_MAP[b as usize]);
                    str::from_utf8(value)?
                } else {
                    ""
                };
                params.insert(name, value);
                if reader.peek() != Some(b'&') {
                    break;
                }
            }

            header_params = Some(GenericParams { params })
        }

        Ok(Uri {
            scheme,
            user,
            host,
            user_param,
            method_param,
            transport_param,
            lr_param,
            maddr_param,
            ttl_param,
            other_params,
            header_params,
        })
    }

    fn parse_status_line(&mut self) -> Result<StatusLine<'a>> {
        let reader = &mut self.reader;
        let _ = tag!(reader, SIPV2);

        space!(reader);
        let digits = digits!(reader);
        space!(reader);

        let status_code = SipStatusCode::from(digits);
        let bytes = until_newline!(reader);

        let rp = str::from_utf8(bytes)?;

        newline!(reader);
        Ok(StatusLine::new(status_code, rp))
    }

    fn parse_request_line(&mut self) -> Result<RequestLine<'a>> {
        let reader = &mut self.reader;
        let b_method = alpha!(reader);
        let method = SipMethod::from(b_method);

        space!(reader);
        let uri = Self::parse_sip_uri(reader)?;
        space!(reader);

        let _ = tag!(reader, SIPV2);
        newline!(reader);

        Ok(RequestLine { method, uri })
    }
}

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl<'a> From<ByteReaderError<'a>> for SipParserError {
    fn from(err: ByteReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{},
                column:{},
                kind:{:?},
                input:'{}'",
                err.pos.line,
                err.pos.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_parse_status_line() {
        let sc_ok = SipStatusCode::Ok;
        let buf = "SIP/2.0 200 OK\r\n".as_bytes();

        assert_eq!(
            SipParser::new(buf).parse_status_line(),
            Ok(StatusLine {
                status_code: sc_ok,
                reason_phrase: sc_ok.reason_phrase()
            })
        );
        let sc_not_found = SipStatusCode::NotFound;
        let buf = "SIP/2.0 404 Not Found\r\n".as_bytes();

        assert_eq!(
            SipParser::new(buf).parse_status_line(),
            Ok(StatusLine {
                status_code: sc_not_found,
                reason_phrase: sc_not_found.reason_phrase()
            })
        );
    }

    #[test]
    fn status_line() {
        let sc_ok = SipStatusCode::Ok;
        let msg = "SIP/2.0 200 OK\r\n".as_bytes();
        let size_of_msg = msg.len();
        let mut counter = 0;
        let now = std::time::Instant::now();
        loop {
            assert_eq!(
                SipParser::new(msg).parse_status_line().unwrap(),
                StatusLine {
                    status_code: sc_ok,
                    reason_phrase: sc_ok.reason_phrase()
                }
            );
            counter += 1;
            if now.elapsed().as_secs() == 1 {
                break;
            }
        }

        println!(
            "{} mbytes per second, count sip messages: {}",
            (size_of_msg * counter) / 1024 / 1024,
            counter
        );
    }

    #[test]
    fn test_req_status_line() {
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let addr: IpAddr = "10.1.1.7".parse().unwrap();
        assert_eq!(
            SipParser::new(msg).parse_request_line(),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: Uri {
                    scheme: Scheme::Sip,
                    user: Some(UserInfo {
                        name: "1000b3",
                        password: None
                    }),
                    host: Host::IpAddr {
                        host: addr,
                        port: Some(8089)
                    },
                    user_param: None,
                    method_param: None,
                    transport_param: None,
                    lr_param: None,
                    maddr_param: None,
                    ttl_param: None,
                    other_params: None,
                    header_params: None,
                }
            })
        );
    }
    #[test]
    fn params() {
        let msg =
            "REGISTER sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15;other=a SIP/2.0\r\n"
                .as_bytes();

        assert_eq!(
            SipParser::new(msg).parse_request_line(),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: Uri {
                    scheme: Scheme::Sip,
                    user: Some(UserInfo {
                        name: "alice",
                        password: None
                    }),
                    host: Host::DomainName {
                        host: "atlanta.com",
                        port: None
                    },
                    user_param: None,
                    method_param: None,
                    transport_param: None,
                    lr_param: None,
                    maddr_param: Some("239.255.255.1"),
                    ttl_param: Some("15"),
                    other_params: Some(GenericParams {
                        params: HashMap::from([("other", "a")])
                    }),
                    header_params: None,
                }
            })
        );
    }
}
