//! SIP Parser

use std::str::{self, Utf8Error};

use reader::newline;
use reader::space;
use reader::until;
use reader::util::{is_newline, is_valid_port};
use reader::Reader;
use reader::{alpha, digits, until_newline};

use crate::headers::Accept;
use crate::headers::AcceptEncoding;
use crate::headers::AcceptLanguage;
use crate::headers::AlertInfo;
use crate::headers::Allow;
use crate::headers::AuthenticationInfo;
use crate::headers::Authorization;
use crate::headers::CSeq;
use crate::headers::CallId;
use crate::headers::Contact;
use crate::headers::ContentDisposition;
use crate::headers::ContentEncoding;
use crate::headers::ContentLength;
use crate::headers::ContentType;
use crate::headers::Date;
use crate::headers::ErrorInfo;
use crate::headers::Expires;
use crate::headers::From;
use crate::headers::Header;
use crate::headers::InReplyTo;
use crate::headers::MaxForwards;
use crate::headers::MimeVersion;
use crate::headers::MinExpires;
use crate::headers::Organization;
use crate::headers::Priority;
use crate::headers::ProxyAuthenticate;
use crate::headers::ProxyAuthorization;
use crate::headers::ProxyRequire;
use crate::headers::RecordRoute;
use crate::headers::ReplyTo;
use crate::headers::Require;
use crate::headers::RetryAfter;
use crate::headers::Route;
use crate::headers::Server;
use crate::headers::SipHeader;
use crate::headers::Subject;
use crate::headers::Supported;
use crate::headers::Timestamp;
use crate::headers::To;
use crate::headers::Unsupported;
use crate::headers::UserAgent;
use crate::headers::Via;
use crate::headers::WWWAuthenticate;
use crate::headers::Warning;
use crate::macros::b_map;
use crate::macros::parse_error;
use crate::macros::parse_param;
use crate::message::HostPort;
use crate::message::Params;
use crate::message::Scheme;
use crate::message::SipMethod;
use crate::message::TransportProtocol;
use crate::message::Uri;
use crate::message::{Host, NameAddr, SipUri, StatusCode};

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use crate::headers::Headers;
use crate::internal::Param;

use crate::message::SipMessage;
use crate::message::StatusLine;
use crate::message::UserInfo;
use crate::message::{RequestLine, SipRequest, SipResponse};
use crate::transport::RequestHeaders;

pub(crate) const SIPV2: &str = "SIP/2.0";
pub(crate) const B_SIPV2: &[u8] = SIPV2.as_bytes();

pub(crate) const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                                    ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                    0123456789";

pub(crate) const UNRESERVED: &[u8] = b"-_.!~*'()%";
pub(crate) const ESCAPED: &[u8] = b"%";
pub(crate) const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
pub(crate) const TOKEN: &[u8] = b"-.!%*_`'~+";
pub(crate) const PASS: &[u8] = b"&=+$,";
pub(crate) const HOST: &[u8] = b"_-.";

// A-Z a-z 0-9 -_.!~*'() &=+$,;?/%
// For reading user part on sip uri.
b_map!(USER_SPEC => ALPHA_NUM, UNRESERVED, USER_UNRESERVED, ESCAPED);
// A-Z a-z 0-9 -_.!~*'() &=+$,%
// For reading password part on sip uri.
b_map!(PASS_SPEC => ALPHA_NUM, UNRESERVED, ESCAPED, PASS);
// A-Z a-z 0-9 -_.
b_map!(HOST_SPEC => ALPHA_NUM, HOST);
// "[]/:&+$"  "-_.!~*'()" "%"
b_map!(PARAM_SPEC => b"[]/:&+$", ALPHA_NUM, UNRESERVED, ESCAPED);
// "[]/?:+$"  "-_.!~*'()" "%"
b_map!(HDR_SPEC => b"[]/?:+$", ALPHA_NUM, UNRESERVED, ESCAPED);
b_map!(TOKEN_SPEC => ALPHA_NUM, TOKEN);

const USER_PARAM: &str = "user";
const METHOD_PARAM: &str = "method";
const TRANSPORT_PARAM: &str = "transport";
const TTL_PARAM: &str = "ttl";
const LR_PARAM: &str = "lr";
const MADDR_PARAM: &str = "maddr";
const SIP: &[u8] = b"sip";
const SIPS: &[u8] = b"sips";

#[inline(always)]
fn is_user(b: &u8) -> bool {
    USER_SPEC[*b as usize]
}

#[inline(always)]
fn is_pass(b: &u8) -> bool {
    PASS_SPEC[*b as usize]
}

#[inline(always)]
fn is_param(b: &u8) -> bool {
    PARAM_SPEC[*b as usize]
}

#[inline(always)]
fn is_hdr(b: &u8) -> bool {
    HDR_SPEC[*b as usize]
}

#[inline(always)]
pub(crate) fn is_host(b: &u8) -> bool {
    HOST_SPEC[*b as usize]
}

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    match b {
        b'A'..=b'Z' => true,
        _ => TOKEN_SPEC[*b as usize],
    }
}

fn parse_uri_param<'a>(reader: &mut Reader<'a>) -> Result<Param<'a>> {
    let Param { name, value } =
        unsafe { Param::parse_unchecked(reader, is_param)? };
    let value = Some(value.unwrap_or(""));

    Ok(Param { name, value })
}

fn parse_hdr_in_uri<'a>(reader: &mut Reader<'a>) -> Result<Param<'a>> {
    Ok(unsafe { Param::parse_unchecked(reader, is_hdr)? })
}

fn read_user_str<'a>(reader: &mut Reader<'a>) -> &'a str {
    unsafe { reader.read_as_str(is_user) }
}

fn read_pass_str<'a>(reader: &mut Reader<'a>) -> &'a str {
    unsafe { reader.read_as_str(is_pass) }
}

fn read_host_str<'a>(reader: &mut Reader<'a>) -> &'a str {
    unsafe { reader.read_as_str(is_host) }
}

fn read_token_str<'a>(reader: &mut Reader<'a>) -> &'a str {
    unsafe { reader.read_as_str(is_token) }
}

/// Parse a buff of bytes into sip message
pub fn parse_sip_msg(buff: &[u8]) -> Result<SipMessage> {
    let reader = &mut Reader::new(buff);
    let mut is_req = false;

    let mut msg = if is_sip_version(reader) {
        let Ok(st_line) = parse_status_line(reader) else {
            return parse_error!("Error parsing 'Status Line'", reader);
        };
        SipMessage::Response(SipResponse {
            st_line,
            headers: Headers::with_capacity(10),
            body: None,
        })
    } else {
        let Ok(req_line) = parse_request_line(reader) else {
            return parse_error!("Error parsing 'Request Line'", reader);
        };
        is_req = true;
        SipMessage::Request(SipRequest {
            req_line,
            headers: Headers::with_capacity(10),
            body: None,
            req_headers: None,
        })
    };

    let mut has_content_type = false;
    let headers = msg.headers_mut();
    let mut via: Vec<Via> = vec![];
    let mut from: Option<From> = None;
    let mut to: Option<To> = None;
    let mut callid: Option<CallId> = None;
    let mut cseq: Option<CSeq> = None;

    macro_rules! parse_header {
        ($header:ident, $reader:ident) => {{
            let Ok(header) = $header::parse($reader) else {
                return parse_error!(
                    format!("Error parsing '{}' header", $header::NAME),
                    $reader
                );
            };
            header
        }};
    }

    'headers: loop {
        let name = parse_token(reader)?;

        if reader.next() != Some(&b':') {
            return parse_error!("Invalid sip header!", reader);
        }
        space!(reader);

        match name {
            ErrorInfo::NAME => {
                let header = parse_header!(ErrorInfo, reader);
                headers.push(Header::ErrorInfo(header));
            }

            Route::NAME => 'route: loop {
                let header = parse_header!(Route, reader);
                headers.push(Header::Route(header));
                let Some(&b',') = reader.peek() else {
                    break 'route;
                };
                reader.next();
            },

            Via::NAME | Via::SHORT_NAME => 'via: loop {
                if is_req {
                    via.push(parse_header!(Via, reader));
                } else {
                    headers.push(Header::Via(parse_header!(Via, reader)));
                }
                let Some(&b',') = reader.peek() else {
                    break 'via;
                };
                reader.next();
            },

            MaxForwards::NAME => {
                let header = parse_header!(MaxForwards, reader);
                headers.push(Header::MaxForwards(header));
            }

            From::NAME | From::SHORT_NAME => {
                if is_req {
                    from = Some(parse_header!(From, reader));
                } else {
                    headers.push(Header::From(parse_header!(From, reader)));
                }
            }

            To::NAME | To::SHORT_NAME => {
                if is_req {
                    to = Some(parse_header!(To, reader));
                } else {
                    headers.push(Header::To(parse_header!(To, reader)));
                }
            }

            CallId::NAME | CallId::SHORT_NAME => {
                if is_req {
                    callid = Some(parse_header!(CallId, reader));
                } else {
                    headers.push(Header::CallId(parse_header!(CallId, reader)));
                }
            }

            CSeq::NAME => {
                if is_req {
                    cseq = Some(parse_header!(CSeq, reader));
                } else {
                    headers.push(Header::CSeq(parse_header!(CSeq, reader)));
                }
            }

            Authorization::NAME => {
                let header = parse_header!(Authorization, reader);
                headers.push(Header::Authorization(header));
            }

            Contact::NAME | Contact::SHORT_NAME => 'contact: loop {
                let header = parse_header!(Contact, reader);
                headers.push(Header::Contact(header));
                let Some(&b',') = reader.peek() else {
                    break 'contact;
                };
                reader.next();
            },

            Expires::NAME => {
                let header = parse_header!(Expires, reader);
                headers.push(Header::Expires(header));
            }

            InReplyTo::NAME => {
                let header = parse_header!(InReplyTo, reader);
                headers.push(Header::InReplyTo(header));
            }

            MimeVersion::NAME => {
                let header = parse_header!(MimeVersion, reader);
                headers.push(Header::MimeVersion(header));
            }

            MinExpires::NAME => {
                let header = parse_header!(MinExpires, reader);
                headers.push(Header::MinExpires(header));
            }

            UserAgent::NAME => {
                let header = parse_header!(UserAgent, reader);
                headers.push(Header::UserAgent(header));
            }

            Date::NAME => {
                let header = parse_header!(Date, reader);
                headers.push(Header::Date(header));
            }

            Server::NAME => {
                let header = parse_header!(Server, reader);
                headers.push(Header::Server(header));
            }

            Subject::NAME | Subject::SHORT_NAME => {
                let header = parse_header!(Subject, reader);
                headers.push(Header::Subject(header));
            }

            Priority::NAME => {
                let header = parse_header!(Priority, reader);
                headers.push(Header::Priority(header));
            }

            ProxyAuthenticate::NAME => {
                let header = parse_header!(ProxyAuthenticate, reader);
                headers.push(Header::ProxyAuthenticate(header));
            }

            ProxyAuthorization::NAME => {
                let header = parse_header!(ProxyAuthorization, reader);
                headers.push(Header::ProxyAuthorization(header));
            }

            ProxyRequire::NAME => {
                let header = parse_header!(ProxyRequire, reader);
                headers.push(Header::ProxyRequire(header));
            }

            ReplyTo::NAME => {
                let header = parse_header!(ReplyTo, reader);
                headers.push(Header::ReplyTo(header));
            }

            ContentLength::NAME | ContentLength::SHORT_NAME => {
                let header = parse_header!(ContentLength, reader);
                headers.push(Header::ContentLength(header));
            }

            ContentEncoding::NAME | ContentEncoding::SHORT_NAME => {
                let header = parse_header!(ContentEncoding, reader);
                headers.push(Header::ContentEncoding(header));
            }

            ContentType::NAME | ContentType::SHORT_NAME => {
                let header = parse_header!(ContentType, reader);
                headers.push(Header::ContentType(header));
                has_content_type = true;
            }

            ContentDisposition::NAME => {
                let header = parse_header!(ContentDisposition, reader);
                headers.push(Header::ContentDisposition(header));
            }

            RecordRoute::NAME => 'rr: loop {
                let header = parse_header!(RecordRoute, reader);
                headers.push(Header::RecordRoute(header));
                let Some(&b',') = reader.peek() else {
                    break 'rr;
                };
                reader.next();
            },

            Require::NAME => {
                let header = parse_header!(Require, reader);
                headers.push(Header::Require(header));
            }

            RetryAfter::NAME => {
                let header = parse_header!(RetryAfter, reader);
                headers.push(Header::RetryAfter(header));
            }

            Organization::NAME => {
                let header = parse_header!(Organization, reader);
                headers.push(Header::Organization(header));
            }

            AcceptEncoding::NAME => {
                let header = parse_header!(AcceptEncoding, reader);
                headers.push(Header::AcceptEncoding(header));
            }

            Accept::NAME => {
                let header = parse_header!(Accept, reader);
                headers.push(Header::Accept(header));
            }

            AcceptLanguage::NAME => {
                let header = parse_header!(AcceptLanguage, reader);
                headers.push(Header::AcceptLanguage(header));
            }

            AlertInfo::NAME => {
                let header = parse_header!(AlertInfo, reader);
                headers.push(Header::AlertInfo(header));
            }

            Allow::NAME => {
                let header = parse_header!(Allow, reader);
                headers.push(Header::Allow(header));
            }

            AuthenticationInfo::NAME => {
                let header = parse_header!(AuthenticationInfo, reader);
                headers.push(Header::AuthenticationInfo(header));
            }

            Supported::NAME | Supported::SHORT_NAME => {
                let header = parse_header!(Supported, reader);
                headers.push(Header::Supported(header));
            }

            Timestamp::NAME => {
                let header = parse_header!(Timestamp, reader);
                headers.push(Header::Timestamp(header));
            }
            Unsupported::NAME => {
                let header = parse_header!(Unsupported, reader);
                headers.push(Header::Unsupported(header));
            }

            WWWAuthenticate::NAME => {
                let header = parse_header!(WWWAuthenticate, reader);
                headers.push(Header::WWWAuthenticate(header));
            }

            Warning::NAME => {
                let header = parse_header!(Warning, reader);
                headers.push(Header::Warning(header));
            }

            _ => {
                let value = Header::parse_header_value_as_str(reader)?;

                headers.push(Header::Other {
                    name: name.into(),
                    value: value.into(),
                });
            }
        };
        if !matches!(reader.peek(), Some(&b'\r') | Some(&b'\n')) {
            return parse_error!("Missing CRLF on header end!", reader);
        }
        reader.read_if(|b| b == &b'\r')?;
        reader.read_if(|b| b == &b'\n')?;

        match reader.peek() {
            Some(b) => {
                if matches!(b, &b'\r' | &b'\n') {
                    break 'headers;
                }
            }
            None => break 'headers,
        }
    }

    if let Some(req) = msg.request_mut() {
        let Some(from) = from else {
            return parse_error!("Missing required 'From' header");
        };
        let Some(to) = to else {
            return parse_error!("Missing required 'To' header");
        };
        let Some(callid) = callid else {
            return parse_error!("Missing required 'Call-ID' header");
        };
        let Some(cseq) = cseq else {
            return parse_error!("Missing required 'CSeq' header");
        };
        req.req_headers = Some(RequestHeaders {
            via,
            from,
            to,
            callid,
            cseq,
        })
    }

    newline!(reader);
    if has_content_type {
        msg.set_body(Some(&buff[reader.idx()..]));
    }

    Ok(msg)
}

fn is_sip_version(reader: &Reader) -> bool {
    match reader.peek_n(4) {
        Some(B_SIPV2) => true,
        _ => false,
    }
}

pub fn parse_sip_v2(reader: &mut Reader) -> Result<()> {
    for b in B_SIPV2 {
        let n = reader.lookahead()?;
        if b != n {
            return parse_error!("Invalid SIP version!");
        }
        reader.next();
    }

    Ok(())
}

fn parse_scheme(reader: &mut Reader) -> Result<Scheme> {
    match until!(reader, &b':') {
        SIP => Ok(Scheme::Sip),
        SIPS => Ok(Scheme::Sips),
        other => parse_error!(format!(
            "Unsupported URI scheme: {}",
            String::from_utf8_lossy(other)
        )),
    }
}

pub(crate) fn parse_user_info(
    reader: &mut Reader,
) -> Result<Option<UserInfo>> {
    let peeked =
        reader.peek_while(|b| b != &b'@' && b != &b'>' && !is_newline(b));

    let Some(&b'@') = peeked else { return Ok(None) };
    let user = read_user_str(reader);
    let pass = match reader.peek() {
        Some(&b':') => {
            reader.next();
            Some(read_pass_str(reader))
        }
        _ => None,
    };
    reader.next();

    Ok(Some(UserInfo::new(user, pass)))
}

fn parse_port(reader: &mut Reader) -> Result<Option<u16>> {
    let Some(&b':') = reader.peek() else {
        return Ok(None);
    };
    reader.next();
    let digits = reader.read_u16()?;
    if is_valid_port(digits) {
        Ok(Some(digits))
    } else {
        parse_error!("Sip Uri Port is invalid!")
    }
}

pub(crate) fn parse_host_port(reader: &mut Reader) -> Result<HostPort> {
    let host = match reader.peek() {
        Some(&b'[') => {
            // Is a Ipv6 host
            reader.next();
            // the '[' and ']' characters are removed from the host
            let host = until!(reader, &b']');
            let host = str::from_utf8(host)?;
            reader.next();

            match host.parse() {
                Ok(addr) => Host::IpAddr(addr),
                Err(_) => return parse_error!("Error parsing Ipv6 HostPort!"),
            }
        }
        _ => {
            let host = read_host_str(reader);

            if host.is_empty() {
                return parse_error!("Can't parse the host!");
            }
            match host.parse() {
                Ok(addr) => Host::IpAddr(addr),
                Err(_) => Host::DomainName(host.into()),
            }
        }
    };

    let port = parse_port(reader)?;
    Ok(HostPort { host, port })
}

fn parse_header_params_in_sip_uri(
    reader: &mut Reader,
) -> Result<Params> {
    reader.next();
    let mut params = Params::new();
    loop {
        // take '&'
        reader.next();
        let Param { name, value } = parse_hdr_in_uri(reader)?;
        let value = value.unwrap_or("".into());
        params.set(name.into(), value.into());

        let Some(b'&') = reader.peek() else { break };
    }
    Ok(params)
}

pub(crate) fn parse_uri(
    reader: &mut Reader,
    parse_params: bool,
) -> Result<Uri> {
    let scheme = parse_scheme(reader)?;
    // take ':'
    reader.next();
    let user = parse_user_info(reader)?;
    let host_port = parse_host_port(reader)?;

    if !parse_params {
        return Ok(Uri::without_params(scheme, user, host_port));
    }

    let mut user_param = None;
    let mut method_param = None;
    let mut transport_param = None;
    let mut ttl_param = None;
    let mut lr_param = None;
    let mut maddr_param = None;

    let params = parse_param!(
        reader,
        parse_uri_param,
        USER_PARAM = user_param,
        METHOD_PARAM = method_param,
        TRANSPORT_PARAM = transport_param,
        TTL_PARAM = ttl_param,
        LR_PARAM = lr_param,
        MADDR_PARAM = maddr_param
    );
    let transport_param = transport_param.map(|s| TransportProtocol::from(&*s));

    let hdr_params = if let Some(&b'?') = reader.peek() {
        Some(parse_header_params_in_sip_uri(reader)?)
    } else {
        None
    };

    Ok(Uri {
        scheme,
        user,
        host_port,
        user_param,
        method_param,
        transport_param,
        ttl_param,
        lr_param,
        maddr_param,
        params,
        hdr_params,
    })
}

pub(crate) fn parse_sip_uri(
    reader: &mut Reader,
    parse_params: bool,
) -> Result<SipUri> {
    space!(reader);
    match reader.peek_n(3) {
        Some(SIP) | Some(SIPS) => {
            let uri = parse_uri(reader, parse_params)?;
            Ok(SipUri::Uri(uri))
        }
        _ => {
            let addr = parse_name_addr(reader)?;
            Ok(SipUri::NameAddr(addr))
        }
    }
}

pub fn parse_name_addr(reader: &mut Reader) -> Result<NameAddr> {
    space!(reader);
    let display = match *(reader.lookahead()?) {
        b'"' => {
            reader.next();
            let display = until!(reader, &b'"');
            reader.next();

            Some(str::from_utf8(display)?)
        }
        b'<' => None,
        _ => {
            let d = parse_token(reader)?;
            space!(reader);

            Some(d)
        }
    };
    space!(reader);
    // must be an '<'
    reader.next();
    let uri = parse_uri(reader, true)?;
    // must be an '>'
    reader.next();

    Ok(NameAddr {
        display: display.map(|s| s.into()),
        uri,
    })
}

pub fn parse_request_line(reader: &mut Reader) -> Result<RequestLine> {
    let method = alpha!(reader);
    let method = SipMethod::from(method);

    space!(reader);
    let uri = parse_uri(reader, true)?;
    space!(reader);

    parse_sip_v2(reader)?;
    newline!(reader);

    Ok(RequestLine { method, uri })
}

pub fn parse_status_line(reader: &mut Reader) -> Result<StatusLine> {
    parse_sip_v2(reader)?;

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let code = StatusCode::from(digits);
    let b = until_newline!(reader);

    let rp = str::from_utf8(b)?;

    newline!(reader);

    Ok(StatusLine::new(code, rp))
}

#[inline]
pub(crate) fn parse_token<'a>(reader: &mut Reader<'a>) -> Result<&'a str> {
    if let Some(&b'"') = reader.peek() {
        reader.next();
        let value = until!(reader, &b'"');
        reader.next();

        Ok(str::from_utf8(value)?)
    } else {
        // is_token ensures that is valid UTF-8
        Ok(read_token_str(reader))
    }
}

/// Error on parsing
#[derive(Debug, PartialEq, Eq)]
pub struct SipParserError {
    /// Message in error
    pub message: String,
}

#[allow(missing_docs)]
impl SipParserError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::convert::From<&str> for SipParserError {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl std::convert::From<String> for SipParserError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::convert::From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

pub enum _SipParseError {
    RequestLine,
    StatusLine,
    Header(&'static str),
}

impl std::convert::From<reader::Error<'_>> for SipParserError {
    fn from(err: reader::Error) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{} column:{} kind:{:?}",
                err.line, err.col, err.kind,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::UriBuilder;

    use super::*;

    macro_rules! st_line {
        ($name:ident,$bytes:expr,$code:expr) => {
            #[test]
            fn $name() {
                let mut reader = Reader::new($bytes);
                let parsed = parse_status_line(&mut reader);
                let parsed = parsed.unwrap();

                assert!(reader.is_eof());
                assert_eq!(parsed.code, $code);
                assert_eq!(&*parsed.rphrase, $code.reason_phrase());
            }
        };
    }

    macro_rules! uri {
        ($name:ident,$bytes:expr,$uri:expr) => {
            #[test]
            fn $name() {
                let mut reader = Reader::new($bytes);
                let parsed = parse_sip_uri(&mut reader, true);
                let parsed = parsed;

                assert_eq!(parsed, $uri);
            }
        };
    }

    uri! {
        test_uri_1,
        b"sip:bob@biloxi.com",
        Ok(SipUri::Uri(
            UriBuilder::new()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host(HostPort::from(Host::DomainName("biloxi.com".into())))
            .get()
        ))
    }

    uri! {
        test_uri_2,
        b"sip:watson@bell-telephone.com",
        Ok(SipUri::Uri(
            UriBuilder::new()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("watson", None))
            .host(HostPort::from(Host::DomainName("bell-telephone.com".into())))
            .get()
        ))
    }

    uri! {
        test_uri_3,
        b"sip:bob@192.0.2.4",
        Ok(SipUri::Uri(
            UriBuilder::new()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host(HostPort::from(Host::IpAddr("192.0.2.4".parse().unwrap())))
            .get()
        ))
    }

    uri! {
        test_uri_4,
        b"sip:user:password@localhost:5060",
        Ok(SipUri::Uri(
            UriBuilder::new()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("user", Some("password")))
            .host(HostPort::new(Host::DomainName("localhost".into()), Some(5060)))
            .get()
        ))
    }

    uri! {
        test_uri_5,
        b"sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15",
        Ok(SipUri::Uri(
            UriBuilder::new()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("alice", None))
            .host(HostPort::from(Host::DomainName("atlanta.com".into())))
            .ttl_param("15")
            .maddr_param("239.255.255.1")
            .get()
        ))
    }

    uri! {
        test_uri_6,
        b"sip:support:pass",
        parse_error!("Failed to parse at line:1 column:13 kind:Num")
    }

    uri! {
        test_uri_7,
        b"sip:support:pass@",
        parse_error!("Can't parse the host!")
    }

    st_line! {
        test_st_line_1,
        b"SIP/2.0 100 Trying\r\n",
        StatusCode::Trying
    }

    st_line! {
        test_st_line_2,
        b"SIP/2.0 180 Ringing\r\n",
        StatusCode::Ringing
    }

    st_line! {
        test_st_line_3,
        b"SIP/2.0 181 Call Is Being Forwarded\r\n",
        StatusCode::CallIsBeingForwarded
    }

    st_line! {
        test_st_line_4,
        b"SIP/2.0 182 Queued\r\n",
        StatusCode::Queued
    }

    st_line! {
        test_st_line_5,
        b"SIP/2.0 183 Session Progress\r\n",
        StatusCode::SessionProgress
    }

    st_line! {
        test_st_line_6,
        b"SIP/2.0 200 OK\r\n",
        StatusCode::Ok
    }

    st_line! {
        test_st_line_7,
        b"SIP/2.0 202 Accepted\r\n",
        StatusCode::Accepted
    }

    st_line! {
        test_st_line_8,
        b"SIP/2.0 300 Multiple Choices\r\n",
        StatusCode::MultipleChoices
    }

    st_line! {
        test_st_line_9,
        b"SIP/2.0 301 Moved Permanently\r\n",
        StatusCode::MovedPermanently
    }

    st_line! {
        test_st_line_10,
        b"SIP/2.0 302 Moved Temporarily\r\n",
        StatusCode::MovedTemporarily
    }

    st_line! {
        test_st_line_11,
        b"SIP/2.0 305 Use Proxy\r\n",
        StatusCode::UseProxy
    }

    st_line! {
        test_st_line_12,
        b"SIP/2.0 380 Alternative Service\r\n",
        StatusCode::AlternativeService
    }

    st_line! {
        test_st_line_13,
        b"SIP/2.0 400 Bad Request\r\n",
        StatusCode::BadRequest
    }

    st_line! {
        test_st_line_14,
        b"SIP/2.0 401 Unauthorized\r\n",
        StatusCode::Unauthorized
    }

    st_line! {
        test_st_line_15,
        b"SIP/2.0 403 Forbidden\r\n",
        StatusCode::Forbidden
    }

    st_line! {
        test_st_line_16,
        b"SIP/2.0 404 Not Found\r\n",
        StatusCode::NotFound
    }

    st_line! {
        test_st_line_17,
        b"SIP/2.0 405 Method Not Allowed\r\n",
        StatusCode::MethodNotAllowed
    }

    st_line! {
        test_st_line_18,
        b"SIP/2.0 406 Not Acceptable\r\n",
        StatusCode::NotAcceptable
    }

    st_line! {
        test_st_line_19,
        b"SIP/2.0 407 Proxy Authentication Required\r\n",
        StatusCode::ProxyAuthenticationRequired
    }

    st_line! {
        test_st_line_20,
        b"SIP/2.0 408 Request Timeout\r\n",
        StatusCode::RequestTimeout
    }

    st_line! {
        test_st_line_21,
        b"SIP/2.0 410 Gone\r\n",
        StatusCode::Gone
    }

    st_line! {
        test_st_line_22,
        b"SIP/2.0 413 Request Entity Too Large\r\n",
        StatusCode::RequestEntityTooLarge
    }

    st_line! {
        test_st_line_23,
        b"SIP/2.0 414 Request-URI Too Long\r\n",
        StatusCode::RequestUriTooLong
    }

    st_line! {
        test_st_line_24,
        b"SIP/2.0 415 Unsupported Media Type\r\n",
        StatusCode::UnsupportedMediaType
    }

    st_line! {
        test_st_line_25,
        b"SIP/2.0 416 Unsupported URI Scheme\r\n",
        StatusCode::UnsupportedUriScheme
    }

    st_line! {
        test_st_line_26,
        b"SIP/2.0 420 Bad Extension\r\n",
        StatusCode::BadExtension
    }

    st_line! {
        test_st_line_27,
        b"SIP/2.0 421 Extension Required\r\n",
        StatusCode::ExtensionRequired
    }

    st_line! {
        test_st_line_28,
        b"SIP/2.0 423 Interval Too Brief\r\n",
        StatusCode::IntervalTooBrief
    }

    st_line! {
        test_st_line_29,
        b"SIP/2.0 480 Temporarily Unavailable\r\n",
        StatusCode::TemporarilyUnavailable
    }

    st_line! {
        test_st_line_30,
        b"SIP/2.0 481 Call/Transaction Does Not Exist\r\n",
        StatusCode::CallOrTransactionDoesNotExist
    }

    st_line! {
        test_st_line_31,
        b"SIP/2.0 482 Loop Detected\r\n",
        StatusCode::LoopDetected
    }

    st_line! {
        test_st_line_32,
        b"SIP/2.0 483 Too Many Hops\r\n",
        StatusCode::TooManyHops
    }

    st_line! {
        test_st_line_33,
        b"SIP/2.0 484 Address Incomplete\r\n",
        StatusCode::AddressIncomplete
    }

    st_line! {
        test_st_line_34,
        b"SIP/2.0 485 Ambiguous\r\n",
        StatusCode::Ambiguous
    }

    st_line! {
        test_st_line_35,
        b"SIP/2.0 486 Busy Here\r\n",
        StatusCode::BusyHere
    }

    st_line! {
        test_st_line_36,
        b"SIP/2.0 487 Request Terminated\r\n",
        StatusCode::RequestTerminated
    }

    st_line! {
        test_st_line_37,
        b"SIP/2.0 488 Not Acceptable Here\r\n",
        StatusCode::NotAcceptableHere
    }

    st_line! {
        test_st_line_38,
        b"SIP/2.0 500 Server Internal Error\r\n",
        StatusCode::ServerInternalError
    }

    st_line! {
        test_st_line_39,
        b"SIP/2.0 501 Not Implemented\r\n",
        StatusCode::NotImplemented
    }

    st_line! {
        test_st_line_40,
        b"SIP/2.0 503 Service Unavailable\r\n",
        StatusCode::ServiceUnavailable
    }

    st_line! {
        test_st_line_41,
        b"SIP/2.0 504 Server Time-out\r\n",
        StatusCode::ServerTimeout
    }

    st_line! {
        test_st_line_42,
        b"SIP/2.0 505 Version Not Supported\r\n",
        StatusCode::VersionNotSupported
    }

    st_line! {
        test_st_line_43,
        b"SIP/2.0 600 Busy Everywhere\r\n",
        StatusCode::BusyEverywhere
    }

    st_line! {
        test_st_line_44,
        b"SIP/2.0 603 Decline\r\n",
        StatusCode::Decline
    }

    st_line! {
        test_st_line_45,
        b"SIP/2.0 604 Does Not Exist Anywhere\r\n",
        StatusCode::DoesNotExistAnywhere
    }

    st_line! {
        test_st_line_46,
        b"SIP/2.0 606 Not Acceptable\r\n",
        StatusCode::NotAcceptableAnywhere
    }
}
