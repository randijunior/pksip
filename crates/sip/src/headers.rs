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
use reader::{space, Reader};
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
    parser::Result, token::Token, uri::Params
};

/// An Header param
pub(crate) type Param<'a> = (&'a str, Option<&'a str>);

/// The tag parameter that is used normaly in [`From`] and [`To`] headers.
const TAG_PARAM: &str = "tag";
/// The q parameter that is used normaly in [`Contact`], [`AcceptEncoding`]
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

pub(crate) fn parse_header_param<'a>(
    reader: &mut Reader<'a>,
) -> Result<Param<'a>> {
    unsafe { parse_param_sip(reader, Token::is_token) }
}

// Parses a `name=value` parameter in a SIP message.
pub(crate) unsafe fn parse_param_sip<'a, F>(
    reader: &mut Reader<'a>,
    func: F,
) -> Result<Param<'a>>
where
    F: Fn(&u8) -> bool,
{
    space!(reader);
    let name = unsafe { reader.read_while_as_str(&func) };
    let Some(&b'=') = reader.peek() else {
        return Ok((name, None));
    };
    reader.next();
    let value = if let Some(&b'"') = reader.peek() {
        reader.next();
        let value = reader::until_byte!(reader, &b'"');
        reader.next();

        str::from_utf8(value)?
    } else {
        unsafe { reader.read_while_as_str(func) }
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

    /// Use the `reader` to parse into this type.
    fn parse(reader: &mut Reader<'a>) -> Result<Self>;

    /// Get this type from `src`
    fn from_bytes(src: &'a [u8]) -> Result<Self> {
        let mut reader = Reader::new(src);

        Self::parse(&mut reader)
    }

    fn parse_as_str(reader: &mut Reader<'a>) -> Result<&'a str> {
        let str = reader::until_newline!(reader);

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
#[derive(Debug, PartialEq, Eq)]
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
}

/// This type reprents an MIME type that indicates an content format.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}

/// The `media-type` that appears in `Accept` and `Content-Type` SIP headers.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

/// This type represents a `q` parameter that is used normaly in [`Contact`], [`AcceptEncoding`]
/// and [`AcceptLanguage`] headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Q(u8, u8);
