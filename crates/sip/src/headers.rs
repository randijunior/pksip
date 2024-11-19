//! SIP Headers types
//!
//! The module provide the [`Headers`] struct that contains an list of [`Header`]
//! and a can be used to manipulating SIP headers.

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
pub use authorization::Authorization;
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
pub use proxy_authenticate::ProxyAuthenticate;
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

use std::str;

use crate::{
    macros::{
        newline, remaing, sip_parse_error, space, until_byte, until_newline,
    },
    parser::Result,
    scanner::Scanner,
    token::{is_token, Token},
    uri::Params,
};

/// An Header param
pub(crate) type Param<'a> = (&'a str, Option<&'a str>);

/// The tag parameter that is used normaly in [`From`] and [`To`] headers.
const TAG_PARAM: &str = "tag";
/// The q parameterthat is used normaly in [`Contact`], [`AcceptEncoding`]
/// and [`AcceptLanguage`] headers.
const Q_PARAM: &str = "q";
/// The expires parameter that is used normaly in [`Contact`] headers.
const EXPIRES_PARAM: &str = "expires";

// Parse the `q` param used in SIP header
fn parse_q(param: &str) -> Option<Q> {
    match param.rsplit_once(".") {
        Some((first, second)) => match (first.parse(), second.parse()) {
            (Ok(a), Ok(b)) => Some(Q(a, b)),
            _ => None,
        },
        None => match param.parse() {
            Ok(n) => Some(Q(n, 0)),
            Err(_) => None,
        },
    }
}

// Parses a `name=value` parameter in a SIP message.
pub(crate) fn parse_header_param<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<Param<'a>> {
    unsafe { parse_param_sip(scanner, is_token) }
}

pub(crate) unsafe fn parse_param_sip<'a, F>(
    scanner: &mut Scanner<'a>,
    func: F,
) -> Result<Param<'a>>
where
    F: Fn(&u8) -> bool,
{
    space!(scanner);
    let name = unsafe { scanner.read_and_convert_to_str(&func) };
    let Some(&b'=') = scanner.peek() else {
        return Ok((name, None));
    };
    scanner.next();
    let value = if let Some(&b'"') = scanner.peek() {
        scanner.next();
        let value = until_byte!(scanner, &b'"');
        scanner.next();

        str::from_utf8(value)?
    } else {
        unsafe { scanner.read_and_convert_to_str(func) }
    };

    Ok((name, Some(value)))
}

/// Trait to parse SIP headers.
///
/// This trait provides methods for header parsing and creation from byte slices.
pub trait SipHeader<'a>: Sized {
    /// The header name in bytes
    const NAME: &'static str;
    /// The header short name(if exists) in bytes
    const SHORT_NAME: Option<&'static str> = None;

    /// Use `bytes` that is a [`Scanner`] instance to parse into this type
    fn parse(scanner: &mut Scanner<'a>) -> Result<Self>;

    /// Get this type from `src`
    fn from_bytes(src: &'a [u8]) -> Result<Self> {
        let mut scanner = Scanner::new(src);

        Self::parse(&mut scanner)
    }

    fn parse_as_str(scanner: &mut Scanner<'a>) -> Result<&'a str> {
        let str = until_newline!(scanner);

        Ok(str::from_utf8(str)?)
    }

    /// Returns `true` if `name` matches this header `name` or `short_name`
    #[inline]
    fn match_name(name: &str) -> bool {
        name == Self::NAME
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }
}

/// SIP headers, as defined in RFC3261.
pub enum Header<'a> {
    /// `Accept` Header
    Accept(Accept<'a>),
    /// `Accept-Enconding` Header
    AcceptEncoding(AcceptEncoding<'a>),
    /// `Accept-Language` Header
    AcceptLanguage(AcceptLanguage<'a>),
    /// `Alert-Info` Header.
    AlertInfo(AlertInfo<'a>),
    /// `Allow` Header
    Allow(Allow<'a>),
    /// `Authentication-Info` Header
    AuthenticationInfo(AuthenticationInfo<'a>),
    /// `Authorization` Header
    Authorization(Authorization<'a>),
    /// `Call-ID` Header
    CallId(CallId<'a>),
    /// `Call-Info` Header
    CallInfo(CallInfo<'a>),
    /// `Contact` Header
    Contact(Contact<'a>),
    /// `Content-Disposition` Header
    ContentDisposition(ContentDisposition<'a>),
    /// `Content-Encoding` Header
    ContentEncoding(ContentEncoding<'a>),
    /// `Content-Language` Header
    ContentLanguage(ContentLanguage<'a>),
    /// `Content-Length` Header
    ContentLength(ContentLength),
    /// `Content-Type` Header
    ContentType(ContentType<'a>),
    /// `CSeq` Header
    CSeq(CSeq<'a>),
    /// `Date` Header
    Date(Date<'a>),
    /// `Error-Info` Header
    ErrorInfo(ErrorInfo<'a>),
    /// `Expires` Header
    Expires(Expires),
    /// `From` Header
    From(From<'a>),
    /// `In-Reply-To` Header
    InReplyTo(InReplyTo<'a>),
    /// `Max-Fowards` Header
    MaxForwards(MaxForwards),
    /// `Min-Expires` Header
    MinExpires(MinExpires),
    /// `MIME-Version` Header
    MimeVersion(MimeVersion<'a>),
    /// `Organization` Header
    Organization(Organization<'a>),
    /// `Priority` Header
    Priority(Priority<'a>),
    /// `Proxy-Authenticate` Header
    ProxyAuthenticate(ProxyAuthenticate<'a>),
    /// `Proxy-Authorization` Header
    ProxyAuthorization(ProxyAuthorization<'a>),
    /// `Proxy-Require` Header
    ProxyRequire(ProxyRequire<'a>),
    /// `Retry-After` Header
    RetryAfter(RetryAfter<'a>),
    /// `Route` Header
    Route(Route<'a>),
    /// `Record-Route` Header
    RecordRoute(RecordRoute<'a>),
    /// `Reply-To` Header
    ReplyTo(ReplyTo<'a>),
    /// `Require` Header
    Require(Require<'a>),
    /// `Server` Header
    Server(Server<'a>),
    /// `Subject` Header
    Subject(Subject<'a>),
    /// `Supported` Header
    Supported(Supported<'a>),
    /// `Timestamp` Header
    Timestamp(Timestamp<'a>),
    /// `To` Header
    To(To<'a>),
    /// `Unsupported` Header
    Unsupported(Unsupported<'a>),
    /// `User-Agent` Header
    UserAgent(UserAgent<'a>),
    /// `Via` Header
    Via(Via<'a>),
    /// `Warning` Header
    Warning(Warning<'a>),
    /// `WWW-Authenticate` Header
    WWWAuthenticate(WWWAuthenticate<'a>),
    /// Other Generic Header
    Other { name: &'a str, value: &'a str },
}

impl<'a> core::convert::From<Vec<Header<'a>>> for Headers<'a> {
    fn from(headers: Vec<Header<'a>>) -> Self {
        Self(headers)
    }
}

/// A set of SIP Headers
///
/// A wrapper over Vec<[`Header`]> that contains the header list
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
///
/// ```
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
    ///
    pub fn find_map<'b, T: 'a, F>(&'b self, f: F) -> Option<&T>
    where
        F: Fn(&'b Header) -> Option<&'a T>,
    {
        self.0.iter().find_map(f)
    }

    /// Returns an iterator over headers
    pub fn iter(&self) -> impl Iterator<Item = &Header<'a>> {
        self.0.iter()
    }

    /// Push an new header
    ///
    /// # Example
    /// ```
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let mut headers = Headers::new();
    ///
    /// headers.push(Header::Expires(Expires::new(10)));
    ///
    /// assert_eq!(headers.len(), 1);
    /// assert!(headers.get(0).is_some());
    pub fn push(&mut self, hdr: Header<'a>) {
        self.0.push(hdr);
    }

    /// Returns the number of headers in the collection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an reference to an header at the index specified
    pub fn get(&self, index: usize) -> Option<&Header> {
        self.0.get(index)
    }

    /// Parse all the sip headers in the buffer of scanner.
    ///
    /// Each parsed header will be pushed into the internal header list. The return value
    /// will be the body of the message in bytes if the [`Header::ContentType`]
    /// is found.
    pub(crate) fn parses_and_return_body(
        &mut self,
        scanner: &mut Scanner<'a>,
    ) -> Result<Option<&'a [u8]>> {
        let mut has_body = false;
        'headers: loop {
            let name = Token::parse(scanner);

            if scanner.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }
            space!(scanner);

            match name {
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
                    let value = Token::parse(scanner);

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

/// This type reprents an MIME type that indicates an content format.
#[derive(Default, Debug, Clone)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}

/// The `media-type` that appears in `Accept` and `Content-Type` SIP headers.
#[derive(Default, Debug, Clone)]
pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}

impl<'a> MediaType<'a> {
    pub fn new(
        mtype: &'a str,
        subtype: &'a str,
        param: Option<Params<'a>>,
    ) -> Self {
        Self {
            mimetype: MimeType { mtype, subtype },
            param,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q(u8, u8);
