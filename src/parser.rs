use crate::headers::auth::authentication_info::AuthenticationInfo;
use crate::headers::auth::authorization::Authorization;
use crate::headers::auth::proxy_authenticate::ProxyAuthenticate;
use crate::headers::auth::proxy_authorization::ProxyAuthorization;
use crate::headers::auth::www_authenticate::WWWAuthenticate;
use crate::headers::capability::accept_encoding::AcceptEncoding;
use crate::headers::capability::accept_language::AcceptLanguage;
use crate::headers::capability::proxy_require::ProxyRequire;
use crate::headers::capability::require::Require;
use crate::headers::capability::supported::Supported;
use crate::headers::capability::unsupported::Unsupported;
use crate::headers::common::call_id::CallId;
use crate::headers::common::cseq::CSeq;
use crate::headers::common::from;
use crate::headers::common::max_fowards::MaxForwards;
use crate::headers::common::to::To;
use crate::headers::control::allow::Allow;
use crate::headers::control::expires::Expires;
use crate::headers::control::min_expires::MinExpires;
use crate::headers::control::reply_to::ReplyTo;
use crate::headers::control::retry_after::RetryAfter;
use crate::headers::control::timestamp::Timestamp;
use crate::headers::info::alert_info::AlertInfo;
use crate::headers::info::date::Date;
use crate::headers::info::error_info::ErrorInfo;
use crate::headers::info::in_reply_to::InReplyTo;
use crate::headers::info::organization::Organization;
use crate::headers::info::priority::Priority;
use crate::headers::info::server::Server;
use crate::headers::info::subject::Subject;
use crate::headers::info::user_agent::UserAgent;
use crate::headers::info::warning::Warning;
use crate::headers::routing::contact::Contact;
use crate::headers::routing::record_route::RecordRoute;
use crate::headers::routing::route::Route;
use crate::headers::routing::via::Via;
use crate::headers::session::accept::Accept;
use crate::headers::session::content_disposition::ContentDisposition;
use crate::headers::session::content_encoding::ContentEncoding;
use crate::headers::session::content_length::ContentLength;
use crate::headers::session::content_type::ContentType;
use crate::headers::session::mime_version::MimeVersion;
use crate::headers::SipHeaderParser;

use crate::{headers::Header, scanner::Scanner};

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::str::Utf8Error;

use crate::headers::SipHeaders;
use crate::scanner::ScannerError;


use crate::macros::find;
use crate::macros::newline;
use crate::macros::peek_while;
use crate::macros::read_while;
use crate::macros::sip_parse_error;
use crate::macros::space;
use crate::macros::until_newline;
use crate::macros::{b_map, remaing};

use crate::msg::SipMsg;
use crate::msg::StatusLine;
use crate::msg::{RequestLine, SipRequest, SipResponse};

use crate::util::is_alphabetic;
use crate::util::is_space;

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

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


b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);


#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC_MAP[*b as usize]
}

#[inline]
pub(crate) fn read_token_utf8<'a>(scanner: &mut Scanner<'a>) -> &'a str {
    let token = read_while!(scanner, is_token);

    // SAFETY: is_token ensures that the bytes are valid utf-8
    unsafe { str::from_utf8_unchecked(token) }
}

pub(crate) fn parse_sip_version_2_0<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<()> {
    let _v = find!(scanner, SIPV2);

    Ok(())
}

fn is_sip_version(scanner: &Scanner) -> bool {
    const SIP: &[u8] = b"SIP";
    let tag = peek_while!(scanner, is_alphabetic);
    let next = scanner.src.get(tag.len());

    next.is_some_and(|next| tag == SIP && (next == &b'/' || is_space(next)))
}

fn parse_headers<'a>(
    scanner: &mut Scanner<'a>,
    headers: &mut SipHeaders<'a>,
) -> Result<Option<&'a [u8]>> {
    let mut has_body = false;
    'headers: loop {
        let name = read_token_utf8(scanner);

        if scanner.next() != Some(&b':') {
            return sip_parse_error!("Invalid sip Header!");
        }
        space!(scanner);

        match name.as_bytes() {
            error_info if ErrorInfo::match_name(error_info) => {
                let error_info = ErrorInfo::parse(scanner)?;
                headers.push_header(Header::ErrorInfo(error_info))
            }
            route if Route::match_name(route) => 'route: loop {
                let route = Route::parse(scanner)?;
                headers.push_header(Header::Route(route));
                let Some(&b',') = scanner.peek() else {
                    break 'route;
                };
                scanner.next();
            },
            via if Via::match_name(via) => 'via: loop {
                let via = Via::parse(scanner)?;
                headers.push_header(Header::Via(via));
                let Some(&b',') = scanner.peek() else {
                    break 'via;
                };
                scanner.next();
            },
            max_fowards if MaxForwards::match_name(max_fowards) => {
                let max_fowards = MaxForwards::parse(scanner)?;
                headers.push_header(Header::MaxForwards(max_fowards))
            }
            from if from::From::match_name(from) => {
                let from = from::From::parse(scanner)?;
                headers.push_header(Header::From(from))
            }
            to if To::match_name(to) => {
                let to = To::parse(scanner)?;
                headers.push_header(Header::To(to))
            }
            cid if CallId::match_name(cid) => {
                let call_id = CallId::parse(scanner)?;
                headers.push_header(Header::CallId(call_id))
            }
            cseq if CSeq::match_name(cseq) => {
                let cseq = CSeq::parse(scanner)?;
                headers.push_header(Header::CSeq(cseq))
            }
            auth if Authorization::match_name(auth) => {
                let auth = Authorization::parse(scanner)?;
                headers.push_header(Header::Authorization(auth))
            }
            contact if Contact::match_name(contact) => 'contact: loop {
                let contact = Contact::parse(scanner)?;
                headers.push_header(Header::Contact(contact));
                let Some(&b',') = scanner.peek() else {
                    break 'contact;
                };
                scanner.next();
            },
            expires if Expires::match_name(expires) => {
                let expires = Expires::parse(scanner)?;
                headers.push_header(Header::Expires(expires));
            }
            in_reply_to if InReplyTo::match_name(in_reply_to) => {
                let in_reply_to = InReplyTo::parse(scanner)?;
                headers.push_header(Header::InReplyTo(in_reply_to));
            }
            mime_version if MimeVersion::match_name(mime_version) => {
                let mime_version = MimeVersion::parse(scanner)?;
                headers.push_header(Header::MimeVersion(mime_version));
            }
            min_expires if MinExpires::match_name(min_expires) => {
                let min_expires = MinExpires::parse(scanner)?;
                headers.push_header(Header::MinExpires(min_expires));
            }
            user_agent if UserAgent::match_name(user_agent) => {
                let user_agent = UserAgent::parse(scanner)?;
                headers.push_header(Header::UserAgent(user_agent))
            }
            date if Date::match_name(date) => {
                let date = Date::parse(scanner)?;
                headers.push_header(Header::Date(date))
            }
            server if Server::match_name(server) => {
                let server = Server::parse(scanner)?;
                headers.push_header(Header::Server(server))
            }
            subject if Subject::match_name(subject) => {
                let subject = Subject::parse(scanner)?;
                headers.push_header(Header::Subject(subject))
            }
            priority if Priority::match_name(priority) => {
                let priority = Priority::parse(scanner)?;
                headers.push_header(Header::Priority(priority))
            }
            proxy_authenticate
                if ProxyAuthenticate::match_name(proxy_authenticate) =>
            {
                let proxy_authenticate = ProxyAuthenticate::parse(scanner)?;
                headers
                    .push_header(Header::ProxyAuthenticate(proxy_authenticate))
            }
            proxy_authorization
                if ProxyAuthorization::match_name(proxy_authorization) =>
            {
                let proxy_authorization = ProxyAuthorization::parse(scanner)?;
                headers.push_header(Header::ProxyAuthorization(
                    proxy_authorization,
                ))
            }
            proxy_require if ProxyRequire::match_name(proxy_require) => {
                let proxy_require = ProxyRequire::parse(scanner)?;
                headers.push_header(Header::ProxyRequire(proxy_require))
            }
            reply_to if ReplyTo::match_name(reply_to) => {
                let reply_to = ReplyTo::parse(scanner)?;
                headers.push_header(Header::ReplyTo(reply_to))
            }
            content_length if ContentLength::match_name(content_length) => {
                let content_length = ContentLength::parse(scanner)?;
                headers.push_header(Header::ContentLength(content_length))
            }
            content_encoding
                if ContentEncoding::match_name(content_encoding) =>
            {
                let content_encoding = ContentEncoding::parse(scanner)?;
                headers.push_header(Header::ContentEncoding(content_encoding))
            }
            content_type if ContentType::match_name(content_type) => {
                let content_type = ContentType::parse(scanner)?;
                has_body = true;
                headers.push_header(Header::ContentType(content_type))
            }
            content_disposition
                if ContentDisposition::match_name(content_disposition) =>
            {
                let content_disposition = ContentDisposition::parse(scanner)?;
                headers.push_header(Header::ContentDisposition(
                    content_disposition,
                ))
            }
            record_route if RecordRoute::match_name(record_route) => {
                'rr: loop {
                    let record_route = RecordRoute::parse(scanner)?;
                    headers.push_header(Header::RecordRoute(record_route));
                    let Some(&b',') = scanner.peek() else {
                        break 'rr;
                    };
                    scanner.next();
                }
            }
            require if Require::match_name(require) => {
                let require = Require::parse(scanner)?;
                headers.push_header(Header::Require(require))
            }
            retry_after if RetryAfter::match_name(retry_after) => {
                let retry_after = RetryAfter::parse(scanner)?;
                headers.push_header(Header::RetryAfter(retry_after))
            }
            organization if Organization::match_name(organization) => {
                let organization = Organization::parse(scanner)?;
                headers.push_header(Header::Organization(organization))
            }
            accept_encoding if AcceptEncoding::match_name(accept_encoding) => {
                let accept_encoding = AcceptEncoding::parse(scanner)?;
                headers.push_header(Header::AcceptEncoding(accept_encoding));
            }
            accept if Accept::match_name(accept) => {
                let accept = Accept::parse(scanner)?;
                headers.push_header(Header::Accept(accept));
            }
            accept_language if AcceptLanguage::match_name(accept_language) => {
                let accept_language = AcceptLanguage::parse(scanner)?;
                headers.push_header(Header::AcceptLanguage(accept_language));
            }
            alert_info if AlertInfo::match_name(alert_info) => {
                let alert_info = AlertInfo::parse(scanner)?;
                headers.push_header(Header::AlertInfo(alert_info));
            }
            allow if Allow::match_name(allow) => {
                let allow = Allow::parse(scanner)?;
                headers.push_header(Header::Allow(allow));
            }
            auth_info if AuthenticationInfo::match_name(auth_info) => {
                let auth_info = AuthenticationInfo::parse(scanner)?;
                headers.push_header(Header::AuthenticationInfo(auth_info));
            }
            supported if Supported::match_name(supported) => {
                let supported = Supported::parse(scanner)?;
                headers.push_header(Header::Supported(supported));
            }
            timestamp if Timestamp::match_name(timestamp) => {
                let timestamp = Timestamp::parse(scanner)?;
                headers.push_header(Header::Timestamp(timestamp));
            }
            user_agent if UserAgent::match_name(user_agent) => {
                let user_agent = UserAgent::parse(scanner)?;
                headers.push_header(Header::UserAgent(user_agent));
            }
            unsupported if Unsupported::match_name(unsupported) => {
                let unsupported = Unsupported::parse(scanner)?;
                headers.push_header(Header::Unsupported(unsupported));
            }
            www_authenticate
                if WWWAuthenticate::match_name(www_authenticate) =>
            {
                let www_authenticate = WWWAuthenticate::parse(scanner)?;
                headers.push_header(Header::WWWAuthenticate(www_authenticate));
            }
            warning if Warning::match_name(warning) => {
                let warning = Warning::parse(scanner)?;
                headers.push_header(Header::Warning(warning));
            }
            _ => {
                let value = until_newline!(scanner);
                let value = str::from_utf8(value)?;

                headers.push_header(Header::Other { name, value });
            }
        };
        newline!(scanner);
        if !scanner.is_eof() {
            continue;
        }
        break 'headers;
    }

    Ok(if has_body {
        Some(remaing!(scanner))
    } else {
        None
    })
}

fn parse_headers_and_body<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<(SipHeaders<'a>, Option<&'a [u8]>)> {
    let mut headers = SipHeaders::new();
    let body = parse_headers(scanner, &mut headers)?;

    Ok((headers, body))
}

/// Parse a buff of bytes into sip message
pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMsg<'a>> {
    let mut scanner = Scanner::new(buff);

    let msg = if !is_sip_version(&scanner) {
        let req_line = RequestLine::parse(&mut scanner)?;
        let (headers, body) = parse_headers_and_body(&mut scanner)?;
        let request = SipRequest::new(req_line, headers, body);

        SipMsg::Request(request)
    } else {
        let status_line = StatusLine::parse(&mut scanner)?;
        let (headers, body) = parse_headers_and_body(&mut scanner)?;
        let response = SipResponse::new(status_line, headers, body);

        SipMsg::Response(response)
    };

    Ok(msg)
}

/// Error on parsing
#[derive(Debug, PartialEq)]
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

impl<'a> From<ScannerError<'a>> for SipParserError {
    fn from(err: ScannerError) -> Self {
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
    use crate::{msg::{SipMethod, SipStatusCode}, uri::{HostPort, Scheme}};

    use super::*;

    #[test]
    fn test_parse_headers() {
        let headers = b"Max-Forwards: 70\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n";

        let mut sip_headers = SipHeaders::new();
        let mut scanner = Scanner::new(headers);
        let body = parse_headers(&mut scanner, &mut sip_headers);
        assert_eq!(body.unwrap(), None);

        let mut iter = sip_headers.iter();
        assert_eq!(
            iter.next().unwrap(),
            &Header::MaxForwards(MaxForwards::new(70))
        );
        assert_eq!(
            iter.next().unwrap(),
            &Header::CallId(CallId::new("843817637684230@998sdasdh09"))
        );
        assert_eq!(
            iter.next().unwrap(),
            &Header::CSeq(CSeq::new(1826, SipMethod::Register))
        );
        assert_eq!(iter.next().unwrap(), &Header::Expires(Expires::new(7200)));
        assert_eq!(
            iter.next().unwrap(),
            &Header::ContentLength(ContentLength::new(0))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_parse_req_line() {
        let req_line = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n";
        let mut scanner = Scanner::new(req_line);
        let parsed = RequestLine::parse(&mut scanner);
        let parsed = parsed.unwrap();

        assert_matches!(parsed, RequestLine { method, uri } => {
            assert_eq!(method, SipMethod::Register);
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(uri.user, None);
            assert_eq!(uri.host, HostPort::DomainName { host: "registrar.biloxi.com", port: None });
        });
    }

    #[test]
    fn status_line() {
        let msg = b"SIP/2.0 200 OK\r\n";
        let mut scanner = Scanner::new(msg);
        let parsed = StatusLine::parse(&mut scanner);
        let parsed = parsed.unwrap();

        assert_matches!(parsed, StatusLine { status_code, reason_phrase } => {
            assert_eq!(status_code, SipStatusCode::Ok);
            assert_eq!(reason_phrase, SipStatusCode::Ok.reason_phrase());
        });
    }
}
