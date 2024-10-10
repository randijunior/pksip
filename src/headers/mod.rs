pub mod accept;
pub mod accept_encoding;
pub mod accept_language;
pub mod alert_info;
pub mod allow;
pub mod authentication_info;
pub mod authorization;
pub mod call_id;
pub mod call_info;
pub mod contact;
pub mod content_disposition;
pub mod content_encoding;
pub mod content_language;
pub mod content_length;
pub mod content_type;
pub mod cseq;
pub mod date;
pub mod error_info;
pub mod expires;
pub mod from;
pub mod in_reply_to;
pub mod max_fowards;
pub mod mime_version;
pub mod min_expires;
pub mod organization;
pub mod priority;
pub mod proxy_authenticate;
pub mod proxy_authorization;
pub mod proxy_require;
pub mod record_route;
pub mod reply_to;
pub mod require;
pub mod retry_after;
pub mod route;
pub mod server;
pub mod subject;
pub mod supported;
pub mod timestamp;
pub mod to;
pub mod unsupported;
pub mod user_agent;
pub mod via;
pub mod warning;
pub mod www_authenticate;

use std::str;

use accept::Accept;
use accept_encoding::AcceptEncoding;
use accept_language::AcceptLanguage;
use alert_info::AlertInfo;
use allow::Allow;
use authentication_info::AuthenticationInfo;
use authorization::{Authorization, Credential, DigestCredential};
pub use call_id::CallId;
use call_info::CallInfo;
use contact::Contact;
use content_disposition::ContentDisposition;
use content_encoding::ContentEncoding;
use content_language::ContentLanguage;
use content_length::ContentLength;
use content_type::ContentType;
use cseq::CSeq;
use date::Date;
use error_info::ErrorInfo;
use expires::Expires;
pub use from::From;
use in_reply_to::InReplyTo;
use max_fowards::MaxForwards;
use mime_version::MimeVersion;
use min_expires::MinExpires;
use organization::Organization;
use priority::Priority;
use proxy_authenticate::{Challenge, DigestChallenge, ProxyAuthenticate};
use proxy_authorization::ProxyAuthorization;
use proxy_require::ProxyRequire;
use record_route::RecordRoute;
use reply_to::ReplyTo;
use require::Require;
use retry_after::RetryAfter;
use route::Route;
use server::Server;
use subject::Subject;
use supported::Supported;
use timestamp::Timestamp;
pub use to::To;
use unsupported::Unsupported;
use user_agent::UserAgent;
pub use via::Via;
use warning::Warning;
use www_authenticate::WWWAuthenticate;


use crate::{
    scanner::Scanner,
    macros::{
        parse_auth_param, read_until_byte, read_while, sip_parse_error, space, until_newline,
    },
    parser::{is_token, Result},
    uri::Params,
};

pub struct OptionTag<'a>(&'a str);

pub(crate) fn parse_generic_param<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<(&'a str, Option<&'a str>)> {
    // take ';' character
    scanner.next();
    space!(scanner);

    let name = read_while!(scanner, is_token);
    let name = unsafe { str::from_utf8_unchecked(name) };
    let value = if scanner.peek() == Some(&b'=') {
        scanner.next();
        let value = read_while!(scanner, is_token);
        Some(unsafe { str::from_utf8_unchecked(value) })
    } else {
        None
    };

    Ok((name, value))
}

pub(crate) trait SipHeaderParser<'a>: Sized {
    const NAME: &'static [u8];
    const SHORT_NAME: Option<&'static [u8]> = None;

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self>;

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME)
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }

    fn parse_q_value(param: Option<&str>) -> Option<f32> {
        if let Some(q_param) = param {
            if let Ok(value) = q_param.parse::<f32>() {
                if (0.0..=1.0).contains(&value) {
                    return Some(value);
                }
            }
            return None;
        }
        None
    }

    fn parse_auth_credential(scanner: &mut Scanner<'a>) -> Result<Credential<'a>> {
        let scheme = match scanner.peek() {
            Some(b'"') => {
                scanner.next();
                let value = read_until_byte!(scanner, b'"');
                scanner.next();
                value
            }
            Some(_) => {
                read_while!(scanner, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => Ok(Credential::Digest(DigestCredential::parse(scanner)?)),
            other => {
                space!(scanner);
                let other = std::str::from_utf8(other)?;
                let name = read_while!(scanner, is_token);
                let name = unsafe { std::str::from_utf8_unchecked(name) };
                let val = parse_auth_param!(scanner);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = scanner.peek() {
                    space!(scanner);
                    let name = read_while!(scanner, is_token);
                    let name = unsafe { std::str::from_utf8_unchecked(name) };
                    let val = parse_auth_param!(scanner);
                    params.set(name, val);
                }

                Ok(Credential::Other {
                    scheme: other,
                    param: params,
                })
            }
        }
    }

    fn parse_auth_challenge(scanner: &mut Scanner<'a>) -> Result<Challenge<'a>> {
        let scheme = match scanner.peek() {
            Some(b'"') => {
                scanner.next();
                let value = read_until_byte!(scanner, b'"');
                scanner.next();
                value
            }
            Some(_) => {
                read_while!(scanner, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => Ok(Challenge::Digest(DigestChallenge::parse(scanner)?)),
            other => {
                space!(scanner);
                let other = std::str::from_utf8(other)?;
                let name = read_while!(scanner, is_token);
                let name = unsafe { std::str::from_utf8_unchecked(name) };
                let val = parse_auth_param!(scanner);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = scanner.peek() {
                    space!(scanner);
                    let name = read_while!(scanner, is_token);
                    let name = unsafe { std::str::from_utf8_unchecked(name) };
                    let val = parse_auth_param!(scanner);
                    params.set(name, val);
                }

                Ok(Challenge::Other {
                    scheme: other,
                    param: params,
                })
            }
        }
    }
}

pub struct SipHeaders<'a> {
    pub(crate) hdrs: Vec<Header<'a>>,
}

impl<'a> SipHeaders<'a> {
    pub fn new() -> Self {
        Self { hdrs: Vec::new() }
    }
    pub fn push_header(&mut self, hdr: Header<'a>) {
        self.hdrs.push(hdr);
    }
}

// Headers, as defined in RFC3261.
pub enum Header<'a> {
    Accept(Accept<'a>),
    AcceptEncoding(AcceptEncoding<'a>),
    AcceptLanguage(AcceptLanguage<'a>),
    AlertInfo(AlertInfo<'a>),
    Allow(Allow<'a>),
    AuthenticationInfo(AuthenticationInfo<'a>),
    Authorization(Authorization<'a>),
    CallId(CallId<'a>),
    CallInfo(CallInfo<'a>),
    Contact(Contact<'a>),
    ContentDisposition(ContentDisposition<'a>),
    ContentEncoding(ContentEncoding<'a>),
    ContentLanguage(ContentLanguage<'a>),
    ContentLength(ContentLength),
    ContentType(ContentType<'a>),
    CSeq(CSeq<'a>),
    Date(Date<'a>),
    ErrorInfo(ErrorInfo<'a>),
    Expires(Expires),
    From(From<'a>),
    InReplyTo(InReplyTo<'a>),
    MaxForwards(MaxForwards),
    MimeVersion(MimeVersion),
    MinExpires(MinExpires),
    Organization(Organization<'a>),
    Priority(Priority<'a>),
    ProxyAuthenticate(ProxyAuthenticate<'a>),
    ProxyAuthorization(ProxyAuthorization<'a>),
    ProxyRequire(ProxyRequire<'a>),
    RecordRoute(RecordRoute<'a>),
    ReplyTo(ReplyTo<'a>),
    Require(Require<'a>),
    RetryAfter(RetryAfter<'a>),
    Route(Route<'a>),
    Server(Server<'a>),
    Subject(Subject<'a>),
    Supported(Supported<'a>),
    Timestamp(Timestamp<'a>),
    To(To<'a>),
    Unsupported(Unsupported<'a>),
    UserAgent(UserAgent<'a>),
    Via(Via<'a>),
    Warning(Warning<'a>),
    WWWAuthenticate(WWWAuthenticate<'a>),
    Other { name: &'a str, value: &'a str },
}
