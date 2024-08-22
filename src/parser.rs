use crate::headers::{self, From as FromHeader, SipHeaderParser, Via};

use crate::{byte_reader::ByteReader, headers::Header};

pub type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::net::IpAddr;
use std::str::FromStr;
use std::str::Utf8Error;

use crate::byte_reader::ByteReaderError;
use crate::headers::via::ViaParams;
use crate::headers::SipHeaders;

use crate::macros::alpha;
use crate::macros::b_map;
use crate::macros::digits;
use crate::macros::find;
use crate::macros::newline;
use crate::macros::peek_while;
use crate::macros::read_while;
use crate::macros::sip_parse_error;
use crate::macros::space;
use crate::macros::until_byte;
use crate::macros::until_newline;

use crate::msg::RequestLine;
use crate::msg::SipMethod;
use crate::msg::SipMsg;
use crate::msg::SipStatusCode;
use crate::msg::StatusLine;

use crate::uri::HostPort;
use crate::uri::Scheme;
use crate::uri::Uri;
use crate::uri::UriParams;
use crate::uri::UserInfo;
use crate::uri::{GenericParams, NameAddr, SipUri};
use crate::util::is_alphabetic;
use crate::util::is_space;
use crate::util::is_valid_port;

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

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
pub(crate) const TRANSPORT_PARAM: &str = "transport";
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

b_map!(VIA_PARAM_SPEC_MAP => b"[:]", ALPHA_NUM, TOKEN);

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
            if matches!(byte, b'@' | b' ' | b'\n' | b'>') {
                matched = Some(byte);
                break;
            }
        }
        matched == Some(b'@')
    }

    fn parse_user(reader: &mut ByteReader<'a>) -> Result<Option<UserInfo<'a>>> {
        if !Self::has_user(reader) {
            return Ok(None);
        }
        let bytes = read_while!(reader, is_user);
        let name = str::from_utf8(bytes)?;
        let mut user = UserInfo {
            name,
            password: None,
        };

        if reader.next() == Some(&b':') {
            let bytes = read_while!(reader, is_pass);
            let bytes = str::from_utf8(bytes)?;
            reader.next();
            user.password = Some(bytes);
        }

        Ok(Some(user))
    }

    pub(crate) fn parse_sip_version(reader: &mut ByteReader<'a>) -> Result<()> {
        let _version = find!(reader, SIPV2);

        Ok(())
    }

    pub(crate) fn parse_sip_uri(reader: &mut ByteReader<'a>) -> Result<SipUri<'a>> {
        space!(reader);
        let peeked = reader.peek();

        match peeked {
            // Nameaddr with quoted display name
            Some(b'"') => {
                reader.next();
                let display = until_byte!(reader, b'"');
                reader.next();
                let display = str::from_utf8(display)?;

                space!(reader);
                
                // must be an '<'
                let Some(&b'<') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!")
                };
                let uri = Self::parse_uri(reader, true)?;
                // must be an '>'
                let Some(&b'>') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!")
                };

                Ok(SipUri::NameAddr(NameAddr { display: Some(display), uri }))

            }
            // NameAddr without display name
            Some(&b'<') => {
                reader.next();
                let uri = Self::parse_uri(reader, true)?;
                reader.next();

                Ok(SipUri::NameAddr(NameAddr { display: None, uri }))
            }
            // SipUri
            Some(_) if reader.peek_n(3) == Some(SCHEME_SIP) => {
                let uri = Self::parse_uri(reader, false)?;
                Ok(SipUri::Uri(uri))
            }
            // Nameaddr with unquoted display name
            Some(_) => {
                let display = read_while!(reader, |b| TOKEN_SPEC_MAP[b as usize]);
                let display = str::from_utf8(display)?;

                space!(reader);
                
                // must be an '<'
                let Some(&b'<') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!")
                };
                let uri = Self::parse_uri(reader, true)?;
                // must be an '>'
                let Some(&b'>') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!")
                };

                Ok(SipUri::NameAddr(NameAddr { display: Some(display), uri }))
            }
            None => {
                todo!()
            }
        }
    }

    pub(crate) fn parse_host(reader: &mut ByteReader<'a>) -> Result<HostPort<'a>> {
        if let Some(_) = reader.read_if(|b| b == b'[') {
            // the '[' and ']' characters are removed from the host
            let host = until_byte!(reader, b']');
            let host = str::from_utf8(host)?;
            reader.next();
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
                    sip_parse_error!("Sip Uri Port is invalid!")
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
            let mut others = GenericParams::new();
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
                    TRANSPORT_PARAM => uri_params.transport = Some(value),
                    TTL_PARAM => uri_params.ttl = Some(value),
                    LR_PARAM => uri_params.lr = Some(value),
                    MADDR_PARAM => uri_params.maddr = Some(value),
                    _ => {
                        others.set(name, value);
                    }
                }
            }
            let params = Some(uri_params);
            let others = if others.is_empty() {
                None
            } else {
                Some(others)
            };

            Ok((params, others))
        } else {
            Ok((None, None))
        }
    }

    fn parse_uri(reader: &mut ByteReader<'a>, parse_params: bool) -> Result<Uri<'a>> {
        let scheme = Self::parse_scheme(reader)?;
        // take ':'
        reader.next();

        let user = Self::parse_user(reader)?;
        let host = Self::parse_host(reader)?;

        if !parse_params {
            return Ok(Uri {
                scheme,
                user,
                host,
                params: None,
                other_params: None,
                header_params: None,
            });
        }
        let (params, other_params) = Self::parse_uri_param(reader)?;

        let mut header_params = None;
        if reader.peek() == Some(&b'?') {
            let mut params = GenericParams::new();
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
                params.set(name, value);
                if reader.peek() != Some(&b'&') {
                    break;
                }
            }

            header_params = Some(params)
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

    pub(crate) fn parse_via_params(
        reader: &mut ByteReader<'a>,
    ) -> Result<(Option<ViaParams<'a>>, Option<GenericParams<'a>>)> {
        if reader.peek() != Some(&b';') {
            return Ok((None, None));
        }
        let mut params = ViaParams::default();
        let mut others = GenericParams::new();
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
                    if !value.is_empty() {
                        match value.parse::<u16>() {
                            Ok(port) if is_valid_port(port) => {
                                params.set_rport(port)
                            }
                            Ok(_) | Err(_) => {
                                return sip_parse_error!(
                                    "Via param rport is invalid!"
                                )
                            }
                        }
                    }
                }
                other => {
                    others.set(name, other);
                }
            }
        }

        let others = if others.is_empty() {
            None
        } else {
            Some(others)
        };

        Ok((Some(params), others))
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
        let uri = Self::parse_uri(reader, true)?;
        space!(reader);

        Self::parse_sip_version(reader)?;
        newline!(reader);

        Ok(RequestLine {
            method,
            uri: SipUri::Uri(uri),
        })
    }

    fn is_sip_request(&self) -> bool {
        const SIP: &[u8] = b"SIP";
        let reader = &self.reader;
        let tag = peek_while!(reader, is_alphabetic);
        let next = reader.src.get(tag.len() + 1);

        next.is_some_and(|next| (next == &b'/' || is_space(*next)) && tag == SIP)
    }

    pub fn parse_headers(&mut self, headers: &mut SipHeaders<'a>) -> Result<()> {
        'headers: loop {
            let reader = &mut self.reader;
            let name = read_while!(reader, |b| TOKEN_SPEC_MAP[b as usize]);

            if reader.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }

            match name {
                via if Via::match_name(via) => 'via: loop {
                    let via = Via::parse(reader)?;
                    headers.push_header(Header::Via(via));
                    let Some(&b',') = reader.peek() else {
                        break 'via;
                    };
                    reader.next();
                },
                to if FromHeader::match_name(to) => {
                    let from = FromHeader::parse(reader)?;
                    headers.push_header(Header::From(from));
                }
                _ => todo!(),
            };
            break 'headers;
        }

        Ok(())
    }
}

pub fn parse_sip_msg<'a>(buff: &'a [u8]) -> Result<SipMsg<'a>> {
    let mut parser = SipParser::new(buff);

    let msg = if parser.is_sip_request() {
        let req_line = parser.parse_request_line()?;
        let mut headers = SipHeaders::new();

        parser.parse_headers(&mut headers);

        todo!()
    } else {
        let status_line = parser.parse_status_line()?;
        let mut headers = SipHeaders::new();

        parser.parse_headers(&mut headers);

        todo!()
    };

    msg
}

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl SipParserError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for SipParserError {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for SipParserError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
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
                "Failed to parse at line:{} column:{} kind:{:?}
                {}",
                err.line,
                err.col,
                err.kind,
                String::from_utf8_lossy(err.src)
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
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let addr: IpAddr = "10.1.1.7".parse().unwrap();
        assert_eq!(
            SipParser::new(msg).parse_request_line(),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: SipUri::Uri(Uri {
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
                })
            })
        );
    }
    /*
    "INVITE sip:user@foo SIP/2.0\n"
    "from: Hi I'm Joe <sip:joe.user@bar.otherdomain.com>;tag=123457890123456\r"
    "To: Fellow User <sip:user@foo.bar.domain.com>\r\n"
    "Call-ID: 12345678901234567890@bar\r\n"
    "Content-Length: 0\r\n"
    "CSeq: 123456 INVITE\n"
    "Contact: <sip:joe@bar> ; q=0.5;expires=3600,sip:user@host;q=0.500\r"
    "  ,sip:user2@host2\n"
    "Content-Type: text/html ; charset=ISO-8859-4\r"
    "Route: <sip:bigbox3.site3.atlanta.com;lr>,\r\n"
    "  <sip:server10.biloxi.com;lr>\r"
    "Record-Route: <sip:server10.biloxi.com>,\r\n" /* multiple routes+folding*/
    "  <sip:bigbox3.site3.atlanta.com;lr>\n"
    "v: SIP/2.0/SCTP bigbox3.site3.atlanta.com;branch=z9hG4bK77ef4c230\n"
    "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKnashds8\n" /* folding. */
    " ;received=192.0.2.1\r\n"
    "Via: SIP/2.0/UDP 10.2.1.1, SIP/2.0/TCP 192.168.1.1\n"
    "Organization: \r"
    "Max-Forwards: 70\n"
    "X-Header: \r\n"        /* empty header */
    "P-Associated-URI:\r\n" /* empty header without space */
    "\r\n"



     */
}
