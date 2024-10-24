//! SIP Headers types
//!
//! The module provide the [`Headers`] struct that contains an list of [`Header`]
//! and a can be used to manipulating SIP headers.

use std::str;

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

pub use accept::Accept;
pub use accept_encoding::AcceptEncoding;
pub use accept_language::AcceptLanguage;
pub use alert_info::AlertInfo;
pub use allow::Allow;
pub use authentication_info::AuthenticationInfo;
pub use authorization::{Authorization, Credential, DigestCredential};
pub use call_id::CallId;
pub use call_info::CallInfo;
pub use contact::Contact;
pub use content_disposition::ContentDisposition;
pub use content_encoding::ContentEncoding;
pub use content_language::ContentLanguage;
pub use content_length::ContentLength;
pub use content_type::ContentType;
pub use cseq::CSeq;
pub use date::Date;
pub use error_info::ErrorInfo;
pub use expires::Expires;
pub use from::From;
pub use in_reply_to::InReplyTo;
pub use max_fowards::MaxForwards;
pub use mime_version::MimeVersion;
pub use min_expires::MinExpires;
pub use organization::Organization;
pub use priority::Priority;
pub use proxy_authenticate::{Challenge, DigestChallenge, ProxyAuthenticate};
pub use proxy_authorization::ProxyAuthorization;
pub use proxy_require::ProxyRequire;
pub use record_route::RecordRoute;
pub use reply_to::ReplyTo;
pub use require::Require;
pub use retry_after::RetryAfter;
pub use route::Route;
pub use server::Server;
pub use subject::Subject;
pub use supported::Supported;
pub use timestamp::Timestamp;
pub use to::To;
pub use unsupported::Unsupported;
pub use user_agent::UserAgent;
pub use via::Via;
pub use warning::Warning;
pub use www_authenticate::WWWAuthenticate;

use crate::{
    bytes::Bytes,
    macros::{
        newline, parse_auth_param, read_until_byte, read_while, remaing,
        sip_parse_error, space, until_newline,
    },
    parser::{self, is_token, Result},
    uri::Params,
};

/// The tag parameter that is used normaly in [`From`] and [`To`] headers.
const TAG_PARAM: &str = "tag";

/// The q parameterthat is used normaly in [`Contact`], [`AcceptEncoding`]
/// and [`AcceptLanguage`] headers.
const Q_PARAM: &str = "q";

/// The expires parameter that is used normaly in [`Contact`] headers.
const EXPIRES_PARAM: &str = "expires";

fn parse_q(param: Option<&str>) -> Option<f32> {
    param
        .and_then(|q| q.parse().ok())
        .filter(|&value| (0.0..=1.0).contains(&value))
}

fn parse_param<'a>(bytes: &mut Bytes<'a>) -> (&'a str, Option<&'a str>) {
    space!(bytes);
    let name = parser::parse_token(bytes);

    let value = if bytes.peek() == Some(&b'=') {
        bytes.next();
        let value = parser::parse_token(bytes);
        Some(value)
    } else {
        None
    };

    (name, value)
}

/// A set of SIP Headers
///
/// A wrapper over Vec<[`Header`]>
///
/// # Examples
/// ```rust
/// # use sip::headers::Headers;
/// # use sip::headers::Header;
/// # use sip::headers::ContentLength;
/// let mut headers = Headers::new();
/// headers.push(Header::ContentLength(ContentLength::new(10)));
///
/// assert_eq!(headers.len(), 1);
/// assert_eq!(
///     headers.get(0),
///     Some(&Header::ContentLength(ContentLength::new(10)))
/// );
///
/// ```
#[derive(Debug, PartialEq)]
pub struct Headers<'a>(Vec<Header<'a>>);

impl<'a> Headers<'a> {
    /// Create a new empty collection of headers
    ///
    /// # Examples
    /// ```
    /// # use sip::headers::Headers;
    /// let mut headers = Headers::new();
    /// ```
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Applies function to the headers and return the first no-none result
    ///
    /// # Examples
    /// ```rust
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let mut headers = Headers::from(vec![
    ///     Header::Expires(Expires::new(10))
    /// ]);
    ///
    /// let expires = headers.find_map(|h| if let Header::Expires(expires) = h {
    ///        Some(expires)
    ///    } else {
    ///        None
    ///    });
    ///
    /// assert!(expires.is_some());
    /// assert_eq!(expires, Some(&Expires::new(10)));
    ///
    pub fn find_map<'b, T: 'a, F>(&'b self, f: F) -> Option<&T>
    where
        F: Fn(&'b Header) -> Option<&'a T>,
    {
        self.0.iter().find_map(f)
    }

    /// Returns an iterator over headers
    ///
    /// # Example
    /// ```rust
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// # use sip::headers::MaxForwards;
    /// let mut headers = Headers::from(vec![
    ///     Header::Expires(Expires::new(10)),
    ///     Header::MaxForwards(MaxForwards::new(70))
    /// ]);
    ///
    /// let mut iter = headers.iter();
    /// assert_eq!(
    ///     iter.next().unwrap(),
    ///     &Header::Expires(Expires::new(10))
    /// );
    /// assert_eq!(
    ///     iter.next().unwrap(),
    ///     &Header::MaxForwards(MaxForwards::new(70))
    /// );
    pub fn iter(&self) -> impl Iterator<Item = &Header<'a>> {
        self.0.iter()
    }

    /// Appends an header
    pub fn push(&mut self, hdr: Header<'a>) {
        self.0.push(hdr);
    }

    /// Returns true if the collection contains the header specified
    pub fn contains(&self, hdr: &Header<'a>) -> bool {
        self.0.contains(hdr)
    }

    /// Returns the number of headers in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an reference to an header at the index specified
    pub fn get(&self, index: usize) -> Option<&Header> {
        self.0.get(index)
    }

    /// Parse all the sip headers in the message
    pub fn parse_headers(
        &mut self,
        bytes: &mut Bytes<'a>,
    ) -> Result<Option<&'a [u8]>> {
        let mut has_body = false;
        'headers: loop {
            let name = parser::parse_token(bytes);

            if bytes.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }
            space!(bytes);

            match name.as_bytes() {
                error_info if ErrorInfo::match_name(error_info) => {
                    let error_info = ErrorInfo::parse(bytes)?;
                    self.push(Header::ErrorInfo(error_info))
                }
                route if Route::match_name(route) => 'route: loop {
                    let route = Route::parse(bytes)?;
                    self.push(Header::Route(route));
                    let Some(&b',') = bytes.peek() else {
                        break 'route;
                    };
                    bytes.next();
                },
                via if Via::match_name(via) => 'via: loop {
                    let via = Via::parse(bytes)?;
                    self.push(Header::Via(via));
                    let Some(&b',') = bytes.peek() else {
                        break 'via;
                    };
                    bytes.next();
                },
                max_fowards if MaxForwards::match_name(max_fowards) => {
                    let max_fowards = MaxForwards::parse(bytes)?;
                    self.push(Header::MaxForwards(max_fowards))
                }
                from if From::match_name(from) => {
                    let from = From::parse(bytes)?;
                    self.push(Header::From(from))
                }
                to if To::match_name(to) => {
                    let to = To::parse(bytes)?;
                    self.push(Header::To(to))
                }
                cid if CallId::match_name(cid) => {
                    let call_id = CallId::parse(bytes)?;
                    self.push(Header::CallId(call_id))
                }
                cseq if CSeq::match_name(cseq) => {
                    let cseq = CSeq::parse(bytes)?;
                    self.push(Header::CSeq(cseq))
                }
                auth if Authorization::match_name(auth) => {
                    let auth = Authorization::parse(bytes)?;
                    self.push(Header::Authorization(auth))
                }
                contact if Contact::match_name(contact) => 'contact: loop {
                    let contact = Contact::parse(bytes)?;
                    self.push(Header::Contact(contact));
                    let Some(&b',') = bytes.peek() else {
                        break 'contact;
                    };
                    bytes.next();
                },
                expires if Expires::match_name(expires) => {
                    let expires = Expires::parse(bytes)?;
                    self.push(Header::Expires(expires));
                }
                in_reply_to if InReplyTo::match_name(in_reply_to) => {
                    let in_reply_to = InReplyTo::parse(bytes)?;
                    self.push(Header::InReplyTo(in_reply_to));
                }
                mime_version if MimeVersion::match_name(mime_version) => {
                    let mime_version = MimeVersion::parse(bytes)?;
                    self.push(Header::MimeVersion(mime_version));
                }
                min_expires if MinExpires::match_name(min_expires) => {
                    let min_expires = MinExpires::parse(bytes)?;
                    self.push(Header::MinExpires(min_expires));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(bytes)?;
                    self.push(Header::UserAgent(user_agent))
                }
                date if Date::match_name(date) => {
                    let date = Date::parse(bytes)?;
                    self.push(Header::Date(date))
                }
                server if Server::match_name(server) => {
                    let server = Server::parse(bytes)?;
                    self.push(Header::Server(server))
                }
                subject if Subject::match_name(subject) => {
                    let subject = Subject::parse(bytes)?;
                    self.push(Header::Subject(subject))
                }
                priority if Priority::match_name(priority) => {
                    let priority = Priority::parse(bytes)?;
                    self.push(Header::Priority(priority))
                }
                proxy_authenticate
                    if ProxyAuthenticate::match_name(proxy_authenticate) =>
                {
                    let proxy_authenticate = ProxyAuthenticate::parse(bytes)?;
                    self.push(Header::ProxyAuthenticate(proxy_authenticate))
                }
                proxy_authorization
                    if ProxyAuthorization::match_name(proxy_authorization) =>
                {
                    let proxy_authorization = ProxyAuthorization::parse(bytes)?;
                    self.push(Header::ProxyAuthorization(proxy_authorization))
                }
                proxy_require if ProxyRequire::match_name(proxy_require) => {
                    let proxy_require = ProxyRequire::parse(bytes)?;
                    self.push(Header::ProxyRequire(proxy_require))
                }
                reply_to if ReplyTo::match_name(reply_to) => {
                    let reply_to = ReplyTo::parse(bytes)?;
                    self.push(Header::ReplyTo(reply_to))
                }
                content_length if ContentLength::match_name(content_length) => {
                    let content_length = ContentLength::parse(bytes)?;
                    self.push(Header::ContentLength(content_length))
                }
                content_encoding
                    if ContentEncoding::match_name(content_encoding) =>
                {
                    let content_encoding = ContentEncoding::parse(bytes)?;
                    self.push(Header::ContentEncoding(content_encoding))
                }
                content_type if ContentType::match_name(content_type) => {
                    let content_type = ContentType::parse(bytes)?;
                    has_body = true;
                    self.push(Header::ContentType(content_type))
                }
                content_disposition
                    if ContentDisposition::match_name(content_disposition) =>
                {
                    let content_disposition = ContentDisposition::parse(bytes)?;
                    self.push(Header::ContentDisposition(content_disposition))
                }
                record_route if RecordRoute::match_name(record_route) => {
                    'rr: loop {
                        let record_route = RecordRoute::parse(bytes)?;
                        self.push(Header::RecordRoute(record_route));
                        let Some(&b',') = bytes.peek() else {
                            break 'rr;
                        };
                        bytes.next();
                    }
                }
                require if Require::match_name(require) => {
                    let require = Require::parse(bytes)?;
                    self.push(Header::Require(require))
                }
                retry_after if RetryAfter::match_name(retry_after) => {
                    let retry_after = RetryAfter::parse(bytes)?;
                    self.push(Header::RetryAfter(retry_after))
                }
                organization if Organization::match_name(organization) => {
                    let organization = Organization::parse(bytes)?;
                    self.push(Header::Organization(organization))
                }
                accept_encoding
                    if AcceptEncoding::match_name(accept_encoding) =>
                {
                    let accept_encoding = AcceptEncoding::parse(bytes)?;
                    self.push(Header::AcceptEncoding(accept_encoding));
                }
                accept if Accept::match_name(accept) => {
                    let accept = Accept::parse(bytes)?;
                    self.push(Header::Accept(accept));
                }
                accept_language
                    if AcceptLanguage::match_name(accept_language) =>
                {
                    let accept_language = AcceptLanguage::parse(bytes)?;
                    self.push(Header::AcceptLanguage(accept_language));
                }
                alert_info if AlertInfo::match_name(alert_info) => {
                    let alert_info = AlertInfo::parse(bytes)?;
                    self.push(Header::AlertInfo(alert_info));
                }
                allow if Allow::match_name(allow) => {
                    let allow = Allow::parse(bytes)?;
                    self.push(Header::Allow(allow));
                }
                auth_info if AuthenticationInfo::match_name(auth_info) => {
                    let auth_info = AuthenticationInfo::parse(bytes)?;
                    self.push(Header::AuthenticationInfo(auth_info));
                }
                supported if Supported::match_name(supported) => {
                    let supported = Supported::parse(bytes)?;
                    self.push(Header::Supported(supported));
                }
                timestamp if Timestamp::match_name(timestamp) => {
                    let timestamp = Timestamp::parse(bytes)?;
                    self.push(Header::Timestamp(timestamp));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(bytes)?;
                    self.push(Header::UserAgent(user_agent));
                }
                unsupported if Unsupported::match_name(unsupported) => {
                    let unsupported = Unsupported::parse(bytes)?;
                    self.push(Header::Unsupported(unsupported));
                }
                www_authenticate
                    if WWWAuthenticate::match_name(www_authenticate) =>
                {
                    let www_authenticate = WWWAuthenticate::parse(bytes)?;
                    self.push(Header::WWWAuthenticate(www_authenticate));
                }
                warning if Warning::match_name(warning) => {
                    let warning = Warning::parse(bytes)?;
                    self.push(Header::Warning(warning));
                }
                _ => {
                    let value = until_newline!(bytes);
                    let value = str::from_utf8(value)?;

                    self.push(Header::Other { name, value });
                }
            };
            newline!(bytes);
            if !bytes.is_eof() {
                continue;
            }
            break 'headers;
        }

        Ok(if has_body {
            Some(remaing!(bytes))
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

pub trait SipHeaderParser<'a>: Sized {
    const NAME: &'static [u8];
    const SHORT_NAME: Option<&'static [u8]> = None;

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self>;

    fn from_bytes(src: &'a [u8]) -> Result<Self> {
        let mut bytes = Bytes::new(src);

        Self::parse(&mut bytes)
    }

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME)
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }

    fn parse_auth_credential(bytes: &mut Bytes<'a>) -> Result<Credential<'a>> {
        let scheme = match bytes.peek() {
            Some(b'"') => {
                bytes.next();
                let value = read_until_byte!(bytes, &b'"');
                bytes.next();
                value
            }
            Some(_) => {
                read_while!(bytes, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => {
                Ok(Credential::Digest(DigestCredential::parse(bytes)?))
            }
            other => {
                space!(bytes);
                let other = std::str::from_utf8(other)?;
                let name = parser::parse_token(bytes);
                let val = parse_auth_param!(bytes);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = bytes.peek() {
                    space!(bytes);
                    let name = parser::parse_token(bytes);
                    let val = parse_auth_param!(bytes);
                    params.set(name, val);
                }

                Ok(Credential::Other {
                    scheme: other,
                    param: params,
                })
            }
        }
    }

    fn parse_auth_challenge(bytes: &mut Bytes<'a>) -> Result<Challenge<'a>> {
        let scheme = match bytes.peek() {
            Some(b'"') => {
                bytes.next();
                let value = read_until_byte!(bytes, &b'"');
                bytes.next();
                value
            }
            Some(_) => {
                read_while!(bytes, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => Ok(Challenge::Digest(DigestChallenge::parse(bytes)?)),
            other => {
                space!(bytes);
                let other = std::str::from_utf8(other)?;
                let name = parser::parse_token(bytes);
                let val = parse_auth_param!(bytes);
                let mut params = Params::new();
                params.set(name, val);

                while let Some(b',') = bytes.peek() {
                    space!(bytes);

                    let name = parser::parse_token(bytes);
                    let val = parse_auth_param!(bytes);
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
