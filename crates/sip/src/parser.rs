//! SIP Parser

use std::net::IpAddr;
use std::str::{self, FromStr, Utf8Error};

use reader::newline;
use reader::space;
use reader::until;
use reader::util::{is_newline, is_valid_port};
use reader::Reader;
use reader::{alpha, digits, until_newline};

use crate::headers::parse_param_sip;
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
use crate::headers::Header;
use crate::headers::InReplyTo;
use crate::headers::MaxForwards;
use crate::headers::MimeVersion;
use crate::headers::MinExpires;
use crate::headers::Organization;
use crate::headers::Param;
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
use crate::macros::parse_param;
use crate::macros::sip_parse_error;
use crate::message::HostPort;
use crate::message::Params;
use crate::message::Scheme;
use crate::message::SipMethod;
use crate::message::Uri;
use crate::message::{Host, NameAddr, SipStatusCode, SipUri};

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use crate::headers::Headers;

use crate::message::SipMessage;
use crate::message::StatusLine;
use crate::message::UserInfo;
use crate::message::{RequestLine, SipRequest, SipResponse};

pub(crate) const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

pub(crate) const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                                    ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                    0123456789";

pub(crate) const UNRESERVED: &[u8] = b"-_.!~*'()%";
pub(crate) const ESCAPED: &[u8] = b"%";
pub(crate) const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
pub(crate) const TOKEN: &[u8] = b"-.!%*_`'~+";
pub(crate) const PASS: &[u8] = b"&=+$,";
pub(crate) const HOST: &[u8] = b"_-.";
pub(crate) const GENERIC_URI: &[u8] = b"#?;:@&=+-_.!~*'()%$,/";

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
b_map!(URI_SPEC => ALPHA_NUM, GENERIC_URI);
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
pub(crate) fn is_uri(b: &u8) -> bool {
    URI_SPEC[*b as usize]
}

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC[*b as usize]
}

fn parse_uri_param<'a>(reader: &mut Reader<'a>) -> Result<Param<'a>> {
    let Param(name, value) = unsafe { parse_param_sip(reader, is_param)? };

    Ok(Param(name, Some(value.unwrap_or(""))))
}

fn parse_hdr_in_uri<'a>(reader: &mut Reader<'a>) -> Result<Param<'a>> {
    Ok(unsafe { parse_param_sip(reader, is_hdr)? })
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
pub fn parse_sip_msg<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
    let mut reader = Reader::new(buff);

    let mut msg = if is_sip_version(&reader) {
        SipMessage::Response(SipResponse {
            st_line: parse_status_line(&mut reader)?,
            headers: Headers::new(),
            body: None,
        })
    } else {
        SipMessage::Request(SipRequest {
            req_line: parse_request_line(&mut reader)?,
            headers: Headers::new(),
            body: None,
        })
    };

    let mut has_content_type = false;
    let reader = &mut reader;

    'headers: loop {
        let name = parse_token(reader)?;

        if reader.next() != Some(&b':') {
            return sip_parse_error!("Invalid sip Header!");
        }
        space!(reader);

        match name {
            error_info if ErrorInfo::match_name(error_info) => {
                let error_info = ErrorInfo::parse(reader)?;
                msg.push_header(Header::ErrorInfo(error_info))
            }

            route if Route::match_name(route) => 'route: loop {
                let route = Route::parse(reader)?;
                msg.push_header(Header::Route(route));
                let Some(&b',') = reader.peek() else {
                    break 'route;
                };
                reader.next();
            },

            via if Via::match_name(via) => 'via: loop {
                let via = Via::parse(reader)?;
                msg.push_header(Header::Via(via));
                let Some(&b',') = reader.peek() else {
                    break 'via;
                };
                reader.next();
            },

            max_fowards if MaxForwards::match_name(max_fowards) => {
                let max_fowards = MaxForwards::parse(reader)?;
                msg.push_header(Header::MaxForwards(max_fowards))
            }

            from if crate::headers::From::match_name(from) => {
                let from = crate::headers::From::parse(reader)?;
                msg.push_header(Header::From(from))
            }

            to if To::match_name(to) => {
                let to = To::parse(reader)?;
                msg.push_header(Header::To(to))
            }

            cid if CallId::match_name(cid) => {
                let call_id = CallId::parse(reader)?;
                msg.push_header(Header::CallId(call_id))
            }

            cseq if CSeq::match_name(cseq) => {
                let cseq = CSeq::parse(reader)?;
                msg.push_header(Header::CSeq(cseq))
            }

            auth if Authorization::match_name(auth) => {
                let auth = Authorization::parse(reader)?;
                msg.push_header(Header::Authorization(auth))
            }

            contact if Contact::match_name(contact) => 'contact: loop {
                let contact = Contact::parse(reader)?;
                msg.push_header(Header::Contact(contact));
                let Some(&b',') = reader.peek() else {
                    break 'contact;
                };
                reader.next();
            },

            expires if Expires::match_name(expires) => {
                let expires = Expires::parse(reader)?;
                msg.push_header(Header::Expires(expires));
            }

            in_reply_to if InReplyTo::match_name(in_reply_to) => {
                let in_reply_to = InReplyTo::parse(reader)?;
                msg.push_header(Header::InReplyTo(in_reply_to));
            }

            mime_version if MimeVersion::match_name(mime_version) => {
                let mime_version = MimeVersion::parse(reader)?;
                msg.push_header(Header::MimeVersion(mime_version));
            }

            min_expires if MinExpires::match_name(min_expires) => {
                let min_expires = MinExpires::parse(reader)?;
                msg.push_header(Header::MinExpires(min_expires));
            }

            user_agent if UserAgent::match_name(user_agent) => {
                let user_agent = UserAgent::parse(reader)?;
                msg.push_header(Header::UserAgent(user_agent))
            }

            date if Date::match_name(date) => {
                let date = Date::parse(reader)?;
                msg.push_header(Header::Date(date))
            }

            server if Server::match_name(server) => {
                let server = Server::parse(reader)?;
                msg.push_header(Header::Server(server))
            }

            subject if Subject::match_name(subject) => {
                let subject = Subject::parse(reader)?;
                msg.push_header(Header::Subject(subject))
            }

            priority if Priority::match_name(priority) => {
                let priority = Priority::parse(reader)?;
                msg.push_header(Header::Priority(priority))
            }

            proxy_authenticate
                if ProxyAuthenticate::match_name(proxy_authenticate) =>
            {
                let proxy_authenticate = ProxyAuthenticate::parse(reader)?;
                msg.push_header(Header::ProxyAuthenticate(proxy_authenticate))
            }

            proxy_authorization
                if ProxyAuthorization::match_name(proxy_authorization) =>
            {
                let proxy_authorization = ProxyAuthorization::parse(reader)?;
                msg.push_header(Header::ProxyAuthorization(proxy_authorization))
            }

            proxy_require if ProxyRequire::match_name(proxy_require) => {
                let proxy_require = ProxyRequire::parse(reader)?;
                msg.push_header(Header::ProxyRequire(proxy_require))
            }

            reply_to if ReplyTo::match_name(reply_to) => {
                let reply_to = ReplyTo::parse(reader)?;
                msg.push_header(Header::ReplyTo(reply_to))
            }

            content_length if ContentLength::match_name(content_length) => {
                let content_length = ContentLength::parse(reader)?;
                msg.push_header(Header::ContentLength(content_length))
            }

            content_encoding
                if ContentEncoding::match_name(content_encoding) =>
            {
                let content_encoding = ContentEncoding::parse(reader)?;
                msg.push_header(Header::ContentEncoding(content_encoding))
            }

            content_type if ContentType::match_name(content_type) => {
                let content_type = ContentType::parse(reader)?;
                has_content_type = true;
                msg.push_header(Header::ContentType(content_type))
            }

            content_disposition
                if ContentDisposition::match_name(content_disposition) =>
            {
                let content_disposition = ContentDisposition::parse(reader)?;
                msg.push_header(Header::ContentDisposition(content_disposition))
            }

            record_route if RecordRoute::match_name(record_route) => {
                'rr: loop {
                    let record_route = RecordRoute::parse(reader)?;
                    msg.push_header(Header::RecordRoute(record_route));
                    let Some(&b',') = reader.peek() else {
                        break 'rr;
                    };
                    reader.next();
                }
            }

            require if Require::match_name(require) => {
                let require = Require::parse(reader)?;
                msg.push_header(Header::Require(require))
            }

            retry_after if RetryAfter::match_name(retry_after) => {
                let retry_after = RetryAfter::parse(reader)?;
                msg.push_header(Header::RetryAfter(retry_after))
            }

            organization if Organization::match_name(organization) => {
                let organization = Organization::parse(reader)?;
                msg.push_header(Header::Organization(organization))
            }

            accept_encoding if AcceptEncoding::match_name(accept_encoding) => {
                let accept_encoding = AcceptEncoding::parse(reader)?;
                msg.push_header(Header::AcceptEncoding(accept_encoding));
            }

            accept if Accept::match_name(accept) => {
                let accept = Accept::parse(reader)?;
                msg.push_header(Header::Accept(accept));
            }

            accept_language if AcceptLanguage::match_name(accept_language) => {
                let accept_language = AcceptLanguage::parse(reader)?;
                msg.push_header(Header::AcceptLanguage(accept_language));
            }

            alert_info if AlertInfo::match_name(alert_info) => {
                let alert_info = AlertInfo::parse(reader)?;
                msg.push_header(Header::AlertInfo(alert_info));
            }

            allow if Allow::match_name(allow) => {
                let allow = Allow::parse(reader)?;
                msg.push_header(Header::Allow(allow));
            }

            auth_info if AuthenticationInfo::match_name(auth_info) => {
                let auth_info = AuthenticationInfo::parse(reader)?;
                msg.push_header(Header::AuthenticationInfo(auth_info));
            }

            supported if Supported::match_name(supported) => {
                let supported = Supported::parse(reader)?;
                msg.push_header(Header::Supported(supported));
            }

            timestamp if Timestamp::match_name(timestamp) => {
                let timestamp = Timestamp::parse(reader)?;
                msg.push_header(Header::Timestamp(timestamp));
            }

            user_agent if UserAgent::match_name(user_agent) => {
                let user_agent = UserAgent::parse(reader)?;
                msg.push_header(Header::UserAgent(user_agent));
            }

            unsupported if Unsupported::match_name(unsupported) => {
                let unsupported = Unsupported::parse(reader)?;
                msg.push_header(Header::Unsupported(unsupported));
            }

            www_authenticate
                if WWWAuthenticate::match_name(www_authenticate) =>
            {
                let www_authenticate = WWWAuthenticate::parse(reader)?;
                msg.push_header(Header::WWWAuthenticate(www_authenticate));
            }

            warning if Warning::match_name(warning) => {
                let warning = Warning::parse(reader)?;
                msg.push_header(Header::Warning(warning));
            }

            _ => {
                let value = parse_token(reader)?;

                msg.push_header(Header::Other { name, value });
            }
        };

        newline!(reader);
        if reader.is_eof() {
            break 'headers;
        }
    }

    if has_content_type {
        msg.set_body(Some(&buff[reader.idx()..]));
    }

    Ok(msg)
}

fn is_sip_version(reader: &Reader) -> bool {
    reader
        .peek_n(4)
        .is_some_and(|sip| sip == SIP && &sip[3] == &b'/')
}

pub fn parse_sip_v2(reader: &mut Reader) -> Result<()> {
    match reader.peek_n(7) {
        Some(SIPV2) => {
            reader.nth(6);
            Ok(())
        }
        _ => sip_parse_error!("Sip Version Invalid"),
    }
}

fn parse_scheme(reader: &mut Reader) -> Result<Scheme> {
    match until!(reader, &b':') {
        SIP => Ok(Scheme::Sip),
        SIPS => Ok(Scheme::Sips),
        // Unsupported URI scheme
        other => sip_parse_error!(format!(
            "Unsupported URI scheme: {}",
            String::from_utf8_lossy(other)
        )),
    }
}

pub(crate) fn parse_user_info<'a>(
    reader: &mut Reader<'a>,
) -> Result<Option<UserInfo<'a>>> {
    let peeked =
        reader.peek_while(|b| b != &b'@' && b != &b'>' && !is_newline(b));

    match peeked {
        Some(&b'@') => {
            let user = read_user_str(reader);
            let pass = match reader.peek() {
                Some(&b':') => {
                    reader.next();
                    Some(read_pass_str(reader))
                }
                _ => None,
            };
            reader.must_read(b'@')?;

            Ok(Some(UserInfo { user, pass }))
        }
        _ => Ok(None),
    }
}

fn parse_port(reader: &mut Reader) -> Result<Option<u16>> {
    let Some(&b':') = reader.peek() else {
        return Ok(None);
    };
    reader.next();
    let digits = reader.read_num()?;
    if is_valid_port(digits) {
        Ok(Some(digits))
    } else {
        sip_parse_error!("Sip Uri Port is invalid!")
    }
}

pub(crate) fn parse_host_port<'a>(
    reader: &mut Reader<'a>,
) -> Result<HostPort<'a>> {
    let host = match reader.peek() {
        Some(&b'[') => {
            // Is a Ipv6 host
            reader.must_read(b'[')?;
            // the '[' and ']' characters are removed from the host
            let host = until!(reader, &b']');
            let host = str::from_utf8(host)?;
            reader.must_read(b']')?;

            match host.parse() {
                Ok(addr) => Host::IpAddr(addr),
                Err(_) => {
                    return sip_parse_error!("Error parsing Ipv6 HostPort!")
                }
            }
        }
        _ => {
            let host = read_host_str(reader);
            match host.parse() {
                Ok(addr) => Host::IpAddr(addr),
                Err(_) => Host::DomainName(host),
            }
        }
    };

    let port = parse_port(reader)?;
    Ok(HostPort { host, port })
}

fn parse_header_params_in_sip_uri<'a>(
    reader: &mut Reader<'a>,
) -> Result<Params<'a>> {
    reader.must_read(b'?')?;
    let mut params = Params::new();
    while let Some(b'?') = reader.peek() {
        // take '?' or '&'
        reader.next();
        let Param(name, value) = parse_hdr_in_uri(reader)?;
        let value = value.unwrap_or("");
        params.set(name, value);
    }
    Ok(params)
}

pub(crate) fn parse_uri<'a>(
    reader: &mut Reader<'a>,
    parse_params: bool,
) -> Result<Uri<'a>> {
    let scheme = parse_scheme(reader)?;
    // take ':'
    reader.must_read(b':')?;
    let user = parse_user_info(reader)?;
    let host = parse_host_port(reader)?;

    if !parse_params {
        return Ok(Uri {
            scheme,
            user,
            host,
            ..Default::default()
        });
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

    let hdr_params = if let Some(&b'?') = reader.peek() {
        Some(parse_header_params_in_sip_uri(reader)?)
    } else {
        None
    };

    Ok(Uri {
        scheme,
        user,
        host,
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

pub(crate) fn parse_sip_uri<'a>(reader: &mut Reader<'a>) -> Result<SipUri<'a>> {
    space!(reader);
    match reader.peek_n(3) {
        Some(SIP) | Some(SIPS) => {
            let uri = parse_uri(reader, false)?;
            Ok(SipUri::Uri(uri))
        }
        _ => {
            let addr = parse_name_addr(reader)?;
            Ok(SipUri::NameAddr(addr))
        }
    }
}

pub fn parse_name_addr<'a>(reader: &mut Reader<'a>) -> Result<NameAddr<'a>> {
    space!(reader);
    let display = match reader.lookahead()? {
        &b'"' => {
            reader.next();
            let display = until!(reader, &b'"');
            reader.must_read(b'"')?;

            Some(str::from_utf8(display)?)
        }
        &b'<' => None,
        _ => {
            let d = parse_token(reader)?;
            space!(reader);

            Some(d)
        }
    };
    space!(reader);
    // must be an '<'
    reader.must_read(b'<')?;
    let uri = parse_uri(reader, true)?;
    // must be an '>'
    reader.must_read(b'>')?;

    Ok(NameAddr { display, uri })
}

pub fn parse_request_line<'a>(
    reader: &mut Reader<'a>,
) -> Result<RequestLine<'a>> {
    let method = alpha!(reader);
    let method = SipMethod::from(method);

    space!(reader);
    let uri = parse_uri(reader, true)?;
    space!(reader);

    parse_sip_v2(reader)?;
    newline!(reader);

    Ok(RequestLine { method, uri })
}

pub fn parse_status_line<'a>(
    reader: &mut Reader<'a>,
) -> Result<StatusLine<'a>> {
    parse_sip_v2(reader)?;

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let code = SipStatusCode::from(digits);
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
#[derive(Debug)]
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

impl<'a> From<reader::Error<'a>> for SipParserError {
    fn from(err: reader::Error) -> Self {
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
                assert_eq!(parsed.rphrase, $code.reason_phrase());
            }
        };
    }


    st_line! {
        test_st_line_1,
        b"SIP/2.0 100 Trying\r\n",
        SipStatusCode::Trying
    }

    st_line! {
        test_st_line_2,
        b"SIP/2.0 180 Ringing\r\n",
        SipStatusCode::Ringing
    }

    st_line! {
        test_st_line_3,
        b"SIP/2.0 181 Call Is Being Forwarded\r\n",
        SipStatusCode::CallIsBeingForwarded
    }

    st_line! {
        test_st_line_4,
        b"SIP/2.0 182 Queued\r\n",
        SipStatusCode::Queued
    }

    st_line! {
        test_st_line_5,
        b"SIP/2.0 183 Session Progress\r\n",
        SipStatusCode::SessionProgress
    }

    st_line! {
        test_st_line_6,
        b"SIP/2.0 200 OK\r\n",
        SipStatusCode::Ok
    }

    st_line! {
        test_st_line_7,
        b"SIP/2.0 202 Accepted\r\n",
        SipStatusCode::Accepted
    }

    st_line! {
        test_st_line_8,
        b"SIP/2.0 300 Multiple Choices\r\n",
        SipStatusCode::MultipleChoices
    }

    st_line! {
        test_st_line_9,
        b"SIP/2.0 301 Moved Permanently\r\n",
        SipStatusCode::MovedPermanently
    }

    st_line! {
        test_st_line_10,
        b"SIP/2.0 302 Moved Temporarily\r\n",
        SipStatusCode::MovedTemporarily
    }

    st_line! {
        test_st_line_11,
        b"SIP/2.0 305 Use Proxy\r\n",
        SipStatusCode::UseProxy
    }

    st_line! {
        test_st_line_12,
        b"SIP/2.0 380 Alternative Service\r\n",
        SipStatusCode::AlternativeService
    }

    st_line! {
        test_st_line_13,
        b"SIP/2.0 400 Bad Request\r\n",
        SipStatusCode::BadRequest
    }

    st_line! {
        test_st_line_14,
        b"SIP/2.0 401 Unauthorized\r\n",
        SipStatusCode::Unauthorized
    }

    st_line! {
        test_st_line_15,
        b"SIP/2.0 403 Forbidden\r\n",
        SipStatusCode::Forbidden
    }

    st_line! {
        test_st_line_16,
        b"SIP/2.0 404 Not Found\r\n",
        SipStatusCode::NotFound
    }

    st_line! {
        test_st_line_17,
        b"SIP/2.0 405 Method Not Allowed\r\n",
        SipStatusCode::MethodNotAllowed
    }

    st_line! {
        test_st_line_18,
        b"SIP/2.0 406 Not Acceptable\r\n",
        SipStatusCode::NotAcceptable
    }

    st_line! {
        test_st_line_19,
        b"SIP/2.0 407 Proxy Authentication Required\r\n",
        SipStatusCode::ProxyAuthenticationRequired
    }

    st_line! {
        test_st_line_20,
        b"SIP/2.0 408 Request Timeout\r\n",
        SipStatusCode::RequestTimeout
    }

    st_line! {
        test_st_line_21,
        b"SIP/2.0 410 Gone\r\n",
        SipStatusCode::Gone
    }

    st_line! {
        test_st_line_22,
        b"SIP/2.0 413 Request Entity Too Large\r\n",
        SipStatusCode::RequestEntityTooLarge
    }

    st_line! {
        test_st_line_23,
        b"SIP/2.0 414 Request-URI Too Long\r\n",
        SipStatusCode::RequestUriTooLong
    }

    st_line! {
        test_st_line_24,
        b"SIP/2.0 415 Unsupported Media Type\r\n",
        SipStatusCode::UnsupportedMediaType
    }

    st_line! {
        test_st_line_25,
        b"SIP/2.0 416 Unsupported URI Scheme\r\n",
        SipStatusCode::UnsupportedUriScheme
    }

    st_line! {
        test_st_line_26,
        b"SIP/2.0 420 Bad Extension\r\n",
        SipStatusCode::BadExtension
    }

    st_line! {
        test_st_line_27,
        b"SIP/2.0 421 Extension Required\r\n",
        SipStatusCode::ExtensionRequired
    }

    st_line! {
        test_st_line_28,
        b"SIP/2.0 423 Interval Too Brief\r\n",
        SipStatusCode::IntervalTooBrief
    }

    st_line! {
        test_st_line_29,
        b"SIP/2.0 480 Temporarily Unavailable\r\n",
        SipStatusCode::TemporarilyUnavailable
    }

    st_line! {
        test_st_line_30,
        b"SIP/2.0 481 Call/Transaction Does Not Exist\r\n",
        SipStatusCode::CallOrTransactionDoesNotExist
    }

    st_line! {
        test_st_line_31,
        b"SIP/2.0 482 Loop Detected\r\n",
        SipStatusCode::LoopDetected
    }

    st_line! {
        test_st_line_32,
        b"SIP/2.0 483 Too Many Hops\r\n",
        SipStatusCode::TooManyHops
    }

    st_line! {
        test_st_line_33,
        b"SIP/2.0 484 Address Incomplete\r\n",
        SipStatusCode::AddressIncomplete
    }

    st_line! {
        test_st_line_34,
        b"SIP/2.0 485 Ambiguous\r\n",
        SipStatusCode::Ambiguous
    }

    st_line! {
        test_st_line_35,
        b"SIP/2.0 486 Busy Here\r\n",
        SipStatusCode::BusyHere
    }

    st_line! {
        test_st_line_36,
        b"SIP/2.0 487 Request Terminated\r\n",
        SipStatusCode::RequestTerminated
    }

    st_line! {
        test_st_line_37,
        b"SIP/2.0 488 Not Acceptable Here\r\n",
        SipStatusCode::NotAcceptableHere
    }

    st_line! {
        test_st_line_38,
        b"SIP/2.0 500 Server Internal Error\r\n",
        SipStatusCode::ServerInternalError
    }

    st_line! {
        test_st_line_39,
        b"SIP/2.0 501 Not Implemented\r\n",
        SipStatusCode::NotImplemented
    }

    st_line! {
        test_st_line_40,
        b"SIP/2.0 503 Service Unavailable\r\n",
        SipStatusCode::ServiceUnavailable
    }

    st_line! {
        test_st_line_41,
        b"SIP/2.0 504 Server Time-out\r\n",
        SipStatusCode::ServerTimeout
    }

    st_line! {
        test_st_line_42,
        b"SIP/2.0 505 Version Not Supported\r\n",
        SipStatusCode::VersionNotSupported
    }

    st_line! {
        test_st_line_43,
        b"SIP/2.0 600 Busy Everywhere\r\n",
        SipStatusCode::BusyEverywhere
    }

    st_line! {
        test_st_line_44,
        b"SIP/2.0 603 Decline\r\n",
        SipStatusCode::Decline
    }

    st_line! {
        test_st_line_45,
        b"SIP/2.0 604 Does Not Exist Anywhere\r\n",
        SipStatusCode::DoesNotExistAnywhere
    }

    st_line! {
        test_st_line_46,
        b"SIP/2.0 606 Not Acceptable\r\n",
        SipStatusCode::NotAcceptableAnywhere
    }
}
