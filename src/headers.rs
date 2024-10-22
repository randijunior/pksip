use common::{call_id::CallId, cseq::CSeq, max_fowards::MaxForwards, to::To};
use std::str;

pub mod auth;
pub mod capability;
pub mod common;
pub mod control;
pub mod info;
pub mod routing;
pub mod session;

use auth::{
    authentication_info::AuthenticationInfo,
    authorization::{Authorization, Credential, DigestCredential},
    proxy_authenticate::{Challenge, DigestChallenge, ProxyAuthenticate},
    proxy_authorization::ProxyAuthorization,
    www_authenticate::WWWAuthenticate,
};
use capability::{
    accept_encoding::AcceptEncoding, accept_language::AcceptLanguage,
    proxy_require::ProxyRequire, require::Require, supported::Supported,
    unsupported::Unsupported,
};
use control::{
    allow::Allow, expires::Expires, min_expires::MinExpires, reply_to::ReplyTo,
    retry_after::RetryAfter, timestamp::Timestamp,
};
use info::{
    alert_info::AlertInfo, call_info::CallInfo, date::Date,
    error_info::ErrorInfo, in_reply_to::InReplyTo, organization::Organization,
    priority::Priority, server::Server, subject::Subject,
    user_agent::UserAgent, warning::Warning,
};
use routing::{
    contact::Contact, record_route::RecordRoute, route::Route, via::Via,
};
use session::{
    accept::Accept, content_disposition::ContentDisposition,
    content_encoding::ContentEncoding, content_language::ContentLanguage,
    content_length::ContentLength, content_type::ContentType,
    mime_version::MimeVersion,
};

use common::from::From;

use crate::{
    macros::{
        newline, parse_auth_param, read_until_byte, read_while, remaing,
        sip_parse_error, space, until_newline,
    },
    parser::{self, is_token, Result},
    scanner::Scanner,
    uri::Params,
};

const TAG_PARAM: &str = "tag";
const Q_PARAM: &str = "q";
const EXPIRES_PARAM: &str = "expires";

#[derive(Debug, PartialEq)]
pub struct Headers<'a>(Vec<Header<'a>>);

impl<'a> Headers<'a> {
    /// Create a new empty collection of headers
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn find<T>(&self) -> Option<&T>
    where
        Header<'a>: AsHeader<T>,
    {
        self.0.iter().find_map(|hdr| hdr.as_header())
    }

    /// Returns an iterator over headers
    pub fn iter(&self) -> impl Iterator<Item = &Header<'a>> {
        self.0.iter()
    }

    /// Appends an header
    pub fn push(&mut self, hdr: Header<'a>) {
        self.0.push(hdr);
    }

    /// Returns the number of headers in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn parse_headers(
        &mut self,
        scanner: &mut Scanner<'a>,
    ) -> Result<Option<&'a [u8]>> {
        let mut has_body = false;
        'headers: loop {
            let name = parser::parse_token(scanner);

            if scanner.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }
            space!(scanner);

            match name.as_bytes() {
                error_info if ErrorInfo::match_name(error_info) => {
                    let error_info = ErrorInfo::parse(scanner)?;
                    self.push(Header::ErrorInfo(error_info))
                }
                route if Route::match_name(route) => 'route: loop {
                    let route = Route::parse(scanner)?;
                    self.push(Header::Route(route));
                    let Some(&b',') = scanner.peek() else {
                        break 'route;
                    };
                    scanner.next();
                },
                via if Via::match_name(via) => 'via: loop {
                    let via = Via::parse(scanner)?;
                    self.push(Header::Via(via));
                    let Some(&b',') = scanner.peek() else {
                        break 'via;
                    };
                    scanner.next();
                },
                max_fowards if MaxForwards::match_name(max_fowards) => {
                    let max_fowards = MaxForwards::parse(scanner)?;
                    self.push(Header::MaxForwards(max_fowards))
                }
                from if From::match_name(from) => {
                    let from = From::parse(scanner)?;
                    self.push(Header::From(from))
                }
                to if To::match_name(to) => {
                    let to = To::parse(scanner)?;
                    self.push(Header::To(to))
                }
                cid if CallId::match_name(cid) => {
                    let call_id = CallId::parse(scanner)?;
                    self.push(Header::CallId(call_id))
                }
                cseq if CSeq::match_name(cseq) => {
                    let cseq = CSeq::parse(scanner)?;
                    self.push(Header::CSeq(cseq))
                }
                auth if Authorization::match_name(auth) => {
                    let auth = Authorization::parse(scanner)?;
                    self.push(Header::Authorization(auth))
                }
                contact if Contact::match_name(contact) => 'contact: loop {
                    let contact = Contact::parse(scanner)?;
                    self.push(Header::Contact(contact));
                    let Some(&b',') = scanner.peek() else {
                        break 'contact;
                    };
                    scanner.next();
                },
                expires if Expires::match_name(expires) => {
                    let expires = Expires::parse(scanner)?;
                    self.push(Header::Expires(expires));
                }
                in_reply_to if InReplyTo::match_name(in_reply_to) => {
                    let in_reply_to = InReplyTo::parse(scanner)?;
                    self.push(Header::InReplyTo(in_reply_to));
                }
                mime_version if MimeVersion::match_name(mime_version) => {
                    let mime_version = MimeVersion::parse(scanner)?;
                    self.push(Header::MimeVersion(mime_version));
                }
                min_expires if MinExpires::match_name(min_expires) => {
                    let min_expires = MinExpires::parse(scanner)?;
                    self.push(Header::MinExpires(min_expires));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(scanner)?;
                    self.push(Header::UserAgent(user_agent))
                }
                date if Date::match_name(date) => {
                    let date = Date::parse(scanner)?;
                    self.push(Header::Date(date))
                }
                server if Server::match_name(server) => {
                    let server = Server::parse(scanner)?;
                    self.push(Header::Server(server))
                }
                subject if Subject::match_name(subject) => {
                    let subject = Subject::parse(scanner)?;
                    self.push(Header::Subject(subject))
                }
                priority if Priority::match_name(priority) => {
                    let priority = Priority::parse(scanner)?;
                    self.push(Header::Priority(priority))
                }
                proxy_authenticate
                    if ProxyAuthenticate::match_name(proxy_authenticate) =>
                {
                    let proxy_authenticate = ProxyAuthenticate::parse(scanner)?;
                    self.push(Header::ProxyAuthenticate(proxy_authenticate))
                }
                proxy_authorization
                    if ProxyAuthorization::match_name(proxy_authorization) =>
                {
                    let proxy_authorization =
                        ProxyAuthorization::parse(scanner)?;
                    self.push(Header::ProxyAuthorization(proxy_authorization))
                }
                proxy_require if ProxyRequire::match_name(proxy_require) => {
                    let proxy_require = ProxyRequire::parse(scanner)?;
                    self.push(Header::ProxyRequire(proxy_require))
                }
                reply_to if ReplyTo::match_name(reply_to) => {
                    let reply_to = ReplyTo::parse(scanner)?;
                    self.push(Header::ReplyTo(reply_to))
                }
                content_length if ContentLength::match_name(content_length) => {
                    let content_length = ContentLength::parse(scanner)?;
                    self.push(Header::ContentLength(content_length))
                }
                content_encoding
                    if ContentEncoding::match_name(content_encoding) =>
                {
                    let content_encoding = ContentEncoding::parse(scanner)?;
                    self.push(Header::ContentEncoding(content_encoding))
                }
                content_type if ContentType::match_name(content_type) => {
                    let content_type = ContentType::parse(scanner)?;
                    has_body = true;
                    self.push(Header::ContentType(content_type))
                }
                content_disposition
                    if ContentDisposition::match_name(content_disposition) =>
                {
                    let content_disposition =
                        ContentDisposition::parse(scanner)?;
                    self.push(Header::ContentDisposition(content_disposition))
                }
                record_route if RecordRoute::match_name(record_route) => {
                    'rr: loop {
                        let record_route = RecordRoute::parse(scanner)?;
                        self.push(Header::RecordRoute(record_route));
                        let Some(&b',') = scanner.peek() else {
                            break 'rr;
                        };
                        scanner.next();
                    }
                }
                require if Require::match_name(require) => {
                    let require = Require::parse(scanner)?;
                    self.push(Header::Require(require))
                }
                retry_after if RetryAfter::match_name(retry_after) => {
                    let retry_after = RetryAfter::parse(scanner)?;
                    self.push(Header::RetryAfter(retry_after))
                }
                organization if Organization::match_name(organization) => {
                    let organization = Organization::parse(scanner)?;
                    self.push(Header::Organization(organization))
                }
                accept_encoding
                    if AcceptEncoding::match_name(accept_encoding) =>
                {
                    let accept_encoding = AcceptEncoding::parse(scanner)?;
                    self.push(Header::AcceptEncoding(accept_encoding));
                }
                accept if Accept::match_name(accept) => {
                    let accept = Accept::parse(scanner)?;
                    self.push(Header::Accept(accept));
                }
                accept_language
                    if AcceptLanguage::match_name(accept_language) =>
                {
                    let accept_language = AcceptLanguage::parse(scanner)?;
                    self.push(Header::AcceptLanguage(accept_language));
                }
                alert_info if AlertInfo::match_name(alert_info) => {
                    let alert_info = AlertInfo::parse(scanner)?;
                    self.push(Header::AlertInfo(alert_info));
                }
                allow if Allow::match_name(allow) => {
                    let allow = Allow::parse(scanner)?;
                    self.push(Header::Allow(allow));
                }
                auth_info if AuthenticationInfo::match_name(auth_info) => {
                    let auth_info = AuthenticationInfo::parse(scanner)?;
                    self.push(Header::AuthenticationInfo(auth_info));
                }
                supported if Supported::match_name(supported) => {
                    let supported = Supported::parse(scanner)?;
                    self.push(Header::Supported(supported));
                }
                timestamp if Timestamp::match_name(timestamp) => {
                    let timestamp = Timestamp::parse(scanner)?;
                    self.push(Header::Timestamp(timestamp));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(scanner)?;
                    self.push(Header::UserAgent(user_agent));
                }
                unsupported if Unsupported::match_name(unsupported) => {
                    let unsupported = Unsupported::parse(scanner)?;
                    self.push(Header::Unsupported(unsupported));
                }
                www_authenticate
                    if WWWAuthenticate::match_name(www_authenticate) =>
                {
                    let www_authenticate = WWWAuthenticate::parse(scanner)?;
                    self.push(Header::WWWAuthenticate(www_authenticate));
                }
                warning if Warning::match_name(warning) => {
                    let warning = Warning::parse(scanner)?;
                    self.push(Header::Warning(warning));
                }
                _ => {
                    let value = until_newline!(scanner);
                    let value = str::from_utf8(value)?;

                    self.push(Header::Other { name, value });
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
}

impl<'a> core::convert::From<Vec<Header<'a>>> for Headers<'a> {
    fn from(headers: Vec<Header<'a>>) -> Self {
        Self(headers)
    }
}

// SIP headers, as defined in RFC3261.
#[derive(Debug, PartialEq)]
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

pub trait AsHeader<T> {
    fn as_header(&self) -> Option<&T>;
}

pub trait SipHeaderParser<'a>: Sized {
    const NAME: &'static [u8];
    const SHORT_NAME: Option<&'static [u8]> = None;

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self>;

    fn from_bytes(src: &'a [u8]) -> Result<Self> {
        let mut scanner = Scanner::new(src);

        Self::parse(&mut scanner)
    }

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME)
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }

    fn parse_auth_credential(
        scanner: &mut Scanner<'a>,
    ) -> Result<Credential<'a>> {
        let scheme = match scanner.peek() {
            Some(b'"') => {
                scanner.next();
                let value = read_until_byte!(scanner, &b'"');
                scanner.next();
                value
            }
            Some(_) => {
                read_while!(scanner, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => {
                Ok(Credential::Digest(DigestCredential::parse(scanner)?))
            }
            other => {
                space!(scanner);
                let other = std::str::from_utf8(other)?;
                let name = parser::parse_token(scanner);
                let val = parse_auth_param!(scanner);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = scanner.peek() {
                    space!(scanner);
                    let name = parser::parse_token(scanner);
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

    fn parse_auth_challenge(
        scanner: &mut Scanner<'a>,
    ) -> Result<Challenge<'a>> {
        let scheme = match scanner.peek() {
            Some(b'"') => {
                scanner.next();
                let value = read_until_byte!(scanner, &b'"');
                scanner.next();
                value
            }
            Some(_) => {
                read_while!(scanner, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => {
                Ok(Challenge::Digest(DigestChallenge::parse(scanner)?))
            }
            other => {
                space!(scanner);
                let other = std::str::from_utf8(other)?;
                let name = parser::parse_token(scanner);
                let val = parse_auth_param!(scanner);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = scanner.peek() {
                    space!(scanner);

                    let name = parser::parse_token(scanner);
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

fn parse_q(param: Option<&str>) -> Option<f32> {
    param
        .and_then(|q| q.parse().ok())
        .filter(|&value| (0.0..=1.0).contains(&value))
}

fn parse_param<'a>(scanner: &mut Scanner<'a>) -> (&'a str, Option<&'a str>) {
    space!(scanner);
    let name = parser::parse_token(scanner);

    let value = if scanner.peek() == Some(&b'=') {
        scanner.next();
        let value = parser::parse_token(scanner);
        Some(value)
    } else {
        None
    };

    (name, value)
}
