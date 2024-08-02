use core::str;
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::str::Utf8Error;

use crate::headers::via::Via;
use crate::headers::via::ViaParams;
use crate::iter::ByteReader;
use crate::iter::ByteReaderError;

use crate::macros::alpha;
use crate::macros::b_map;
use crate::macros::digits;
use crate::macros::find;
use crate::macros::newline;
use crate::macros::read_while;
use crate::macros::sip_parse_error;
use crate::macros::space;
use crate::macros::until_byte;
use crate::macros::until_newline;

use crate::msg::RequestLine;
use crate::msg::SipMethod;
use crate::msg::SipStatusCode;
use crate::msg::StatusLine;

use crate::msg::Transport;
use crate::uri::GenericParams;
use crate::uri::HostPort;
use crate::uri::Scheme;
use crate::uri::Uri;
use crate::uri::UriParams;
use crate::uri::UserInfo;
use crate::util::is_valid_port;

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

type Result<T> = std::result::Result<T, SipParserError>;

const ALPHA_NUM: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const UNRESERVED: &[u8] = b"-_.!~*'()%";
const ESCAPED: &[u8] = b"%";
const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
const TOKEN: &[u8] = b"-.!%*_`'~+";
const PASS: &[u8] = b"&=+$,";
const HOST: &[u8] = b"_-.";

pub(crate) const USER_PARAM: &str = "user";
pub(crate) const METHOD_PARAM: &str = "method";
pub(crate) const TANSPORT_PARAM: &str = "transport";
pub(crate) const TTL_PARAM: &str = "ttl";
pub(crate) const LR_PARAM: &str = "lr";
pub(crate) const MADDR_PARAM: &str = "maddr";
pub(crate) const BRANCH_PARAM: &str = "branch";
pub(crate) const RPORT_PARAM: &str = "rport";
pub(crate) const RECEIVED_PARAM: &str = "received";

pub(crate) const SCHEME_SIP: &[u8] = b"sip";
pub(crate) const SCHEME_SIPS: &[u8] = b"sips";

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

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

b_map!(VIA_PARAM_SPEC_MAP => b"[:]", TOKEN_SPEC_MAP);

#[inline(always)]
fn is_user(b: u8) -> bool {
    USER_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_pass(b: u8) -> bool {
    PASS_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_param(b: u8) -> bool {
    PARAM_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_hdr(b: u8) -> bool {
    HDR_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_via_param(b: u8) -> bool {
    VIA_PARAM_SPEC_MAP[b as usize]
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
            SCHEME_SIP => Ok(Scheme::Sip),
            SCHEME_SIPS => Ok(Scheme::Sips),
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

    fn parse_user(
        reader: &mut ByteReader<'a>,
    ) -> Result<Option<UserInfo<'a>>> {
        if Self::has_user(reader) {
            let bytes = read_while!(reader, is_user);
            let name = str::from_utf8(bytes)?;
            return if reader.next() == Some(&b':') {
                let bytes = read_while!(reader, is_pass);
                let bytes = str::from_utf8(bytes)?;
                reader.next();

                Ok(Some(UserInfo {
                    name,
                    password: Some(bytes),
                }))
            } else {
                Ok(Some(UserInfo {
                    name,
                    password: None,
                }))
            };
        }

        Ok(None)
    }

    fn parse_sip_version(reader: &mut ByteReader<'a>) -> Result<()> {
        let _version = find!(reader, SIPV2);

        Ok(())
    }

    fn parse_host(reader: &mut ByteReader<'a>) -> Result<HostPort<'a>> {
        if let Some(_) = reader.read_if(|b| b == b'[') {
            // the '[' and ']' characters are removed from the host
            reader.next();
            let host = until_byte!(reader, b']');
            reader.next();
            let host = str::from_utf8(host)?;
            return if let Ok(host) = host.parse() {
                reader.next();
                Ok(HostPort::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(reader)?,
                })
            } else {
                sip_parse_error!("Error parsing Ipv6 HostPort!")
            };
        }
        let host = read_while!(reader, |b| HOST_SPEC_MAP[b as usize]);
        let host = str::from_utf8(host)?;
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(HostPort::IpAddr {
                host: addr,
                port: Self::parse_port(reader)?,
            })
        } else {
            Ok(HostPort::DomainName {
                host,
                port: Self::parse_port(reader)?,
            })
        }
    }

    fn parse_port(reader: &mut ByteReader) -> Result<Option<u16>> {
        if let Some(_) = reader.read_if(|b| b == b':') {
            let digits = digits!(reader);
            let digits = str::from_utf8(digits)?;
            match digits.parse::<u16>() {
                Ok(port) if is_valid_port(port) => Ok(Some(port)),
                Ok(_) | Err(_) => {
                    sip_parse_error!("Port is invalid!")
                }
            }
        } else {
            Ok(None)
        }
    }

    fn parse_uri_param(
        reader: &mut ByteReader<'a>,
    ) -> Result<(Option<UriParams<'a>>, Option<GenericParams<'a>>)> {
        if reader.peek() == Some(&b';') {
            let mut others = HashMap::new();
            let mut uri_params = UriParams::default();
            while let Some(&b';') = reader.peek() {
                reader.next();
                let name = read_while!(reader, is_param);
                let name = str::from_utf8(name)?;
                let value = if reader.peek() == Some(&b'=') {
                    reader.next();
                    let value = read_while!(reader, is_param);
                    str::from_utf8(value)?
                } else {
                    ""
                };
                match name {
                    USER_PARAM => uri_params.user = Some(value),
                    METHOD_PARAM => uri_params.method = Some(value),
                    TANSPORT_PARAM => uri_params.transport = Some(value),
                    TTL_PARAM => uri_params.ttl = Some(value),
                    LR_PARAM => uri_params.lr = Some(value),
                    MADDR_PARAM => uri_params.maddr = Some(value),
                    _ => {
                        others.insert(name, value);
                    }
                }
            }
            let params = Some(uri_params);
            let others = if others.is_empty() {
                None
            } else {
                Some(GenericParams::new(others))
            };

            Ok((params, others))
        } else {
            Ok((None, None))
        }
    }

    fn parse_sip_uri(reader: &mut ByteReader<'a>) -> Result<Uri<'a>> {
        let scheme = Self::parse_scheme(reader)?;
        // take ':'
        reader.next();

        let user = Self::parse_user(reader)?;
        let host = Self::parse_host(reader)?;
        let (params, other_params) = Self::parse_uri_param(reader)?;

        let mut header_params = None;
        if reader.peek() == Some(&b'?') {
            let mut params = HashMap::new();
            loop {
                // take '?' or '&'
                reader.next();
                let name = read_while!(reader, is_hdr);
                let name = str::from_utf8(name)?;
                let value = if reader.peek() == Some(&b'=') {
                    reader.next();
                    let value = read_while!(reader, is_hdr);
                    str::from_utf8(value)?
                } else {
                    ""
                };
                params.insert(name, value);
                if reader.peek() != Some(&b'&') {
                    break;
                }
            }

            header_params = Some(GenericParams::new(params))
        }

        Ok(Uri {
            scheme,
            user,
            host,
            params,
            other_params,
            header_params,
        })
    }

    fn parse_via_params(
        reader: &mut ByteReader<'a>,
    ) -> Result<Option<ViaParams<'a>>> {
        let mut params = ViaParams::default();
        while let Some(&b';') = reader.peek() {
            reader.next();
            let name = read_while!(reader, is_via_param);
            let name = str::from_utf8(name)?;
            let mut value = "";
            if let Some(&b'=') = reader.peek() {
                reader.next();
                let v = read_while!(reader, is_via_param);
                value = str::from_utf8(v)?;
            }
            match name {
                BRANCH_PARAM => params.set_branch(value),
                TTL_PARAM => params.set_ttl(value),
                MADDR_PARAM => params.set_maddr(value),
                RECEIVED_PARAM => params.set_received(value),
                RPORT_PARAM => {
                    todo!()
                }
                _ => (),
            }
        }
        Ok(None)
    }

    fn parse_via_hdr(reader: &mut ByteReader<'a>) -> Result<Via<'a>> {
        Self::parse_sip_version(reader)?;
        if reader.next() != Some(&b'/') {
            return Err(SipParserError {
                message: format!(
                    "Failed to parse at line:{} column:{}",
                    reader.col(),
                    reader.line()
                ),
            });
        }
        let bytes = until_byte!(reader, b' ');
        let transport = Transport::from(bytes);
        let sent_by = Self::parse_host(reader)?;
        let params = Self::parse_via_params(reader)?;

        todo!()
    }

    fn parse_status_line(&mut self) -> Result<StatusLine<'a>> {
        let reader = &mut self.reader;
        Self::parse_sip_version(reader)?;

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

        Self::parse_sip_version(reader)?;
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
                err.line,
                err.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use crate::uri::Uri;

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
                SipParser::new(msg).parse_status_line(),
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
            (size_of_msg * counter) / 1024 / 1024,
            counter
        );
    }

    #[test]
    fn test_req_status_line() {
        let msg =
            "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
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
                    host: HostPort::IpAddr {
                        host: addr,
                        port: Some(8089)
                    },
                    params: None,
                    other_params: None,
                    header_params: None,
                }
            })
        );
    }
    #[test]
    fn params() {
        let msg = "REGISTER sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15;other=a SIP/2.0\r\n"
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
                    host: HostPort::DomainName {
                        host: "atlanta.com",
                        port: None
                    },
                    params: Some(UriParams {
                        user: None,
                        method: None,
                        transport: None,
                        lr: None,
                        maddr: Some("239.255.255.1"),
                        ttl: Some("15")
                    }),
                    other_params: Some(GenericParams {
                        inner: HashMap::from([("other", "a")])
                    }),
                    header_params: None,
                }
            })
        );
    }
}
