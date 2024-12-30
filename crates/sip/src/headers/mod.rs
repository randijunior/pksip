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

use core::fmt;
use reader::{space, Reader};
use std::{
    iter::{Filter, FilterMap},
    str,
};

use crate::parser::{parse_token, Result};

/// The tag parameter that is used normaly in [`From`] and [`To`] headers.
const TAG_PARAM: &str = "tag";
/// The q parameter that is used normaly in [`Contact`], [`AcceptEncoding`]
/// and [`AcceptLanguage`] headers.
const Q_PARAM: &str = "q";
/// The expires parameter that is used normaly in [`Contact`] headers.
const EXPIRES_PARAM: &str = "expires";

/// Trait to parse SIP headers.
///
/// This trait provides methods for header parsing and creation from byte slices.
pub trait SipHeader<'a>: Sized {
    /// The header name in bytes
    const NAME: &'static str;
    /// The header short name(if exists) in bytes
    const SHORT_NAME: &'static str =
        panic!("This header not have a short name!");

    /// Use the `reader` to parse into this type.
    fn parse(reader: &mut Reader<'a>) -> Result<Self>;

    /// Get this type from `src`
    fn from_bytes(src: &'a [u8]) -> Result<Self> {
        let mut reader = Reader::new(src);

        Self::parse(&mut reader)
    }

    /// See the documentation for [`Header::parse_header_value_as_str`]
    fn parse_as_str(reader: &mut Reader<'a>) -> Result<&'a str> {
        Header::parse_header_value_as_str(reader)
    }
}

/// An SIP Header.
///
/// This enum contain the SIP headers, as defined in `RFC3261`.
///
/// # Examples
///
/// ```
/// use sip::headers::{Header, ContentLength, CallId};
///
/// let c_len = Header::ContentLength(ContentLength::new(10));
/// let cid = Header::CallId(CallId::new("bs9ki9iqbee8k5kal8mpqb"));
///
/// assert_eq!(Header::from_bytes(b"Content-Length: 10"), Ok(c_len));
/// assert_eq!(Header::from_bytes(b"Call-ID: bs9ki9iqbee8k5kal8mpqb"), Ok(cid));
/// ```
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
    Allow(Allow),
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
    CSeq(CSeq),
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

impl fmt::Display for Header<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Header::Via(h) => write!(f, "{}: {}", Via::NAME, h),
            Header::Accept(h) => write!(f, "{}: {}", Accept::NAME, h),
            Header::AcceptEncoding(h) => {
                write!(f, "{}: {}", AcceptEncoding::NAME, h)
            }
            Header::AcceptLanguage(h) => {
                write!(f, "{}: {}", AcceptLanguage::NAME, h)
            }
            Header::AlertInfo(h) => write!(f, "{}: {}", AlertInfo::NAME, h),
            Header::Allow(h) => write!(f, "{}: {}", Allow::NAME, h),
            Header::AuthenticationInfo(h) => {
                write!(f, "{}: {}", AuthenticationInfo::NAME, h)
            }
            Header::Authorization(h) => {
                write!(f, "{}: {}", Authorization::NAME, h)
            }
            Header::CallId(h) => write!(f, "{}: {}", CallId::NAME, h),
            Header::CallInfo(h) => write!(f, "{}: {}", CallInfo::NAME, h),
            Header::Contact(h) => write!(f, "{}: {}", Contact::NAME, h),
            Header::ContentDisposition(h) => {
                write!(f, "{}: {}", ContentDisposition::NAME, h)
            }
            Header::ContentEncoding(h) => {
                write!(f, "{}: {}", ContentEncoding::NAME, h)
            }
            Header::ContentLanguage(h) => {
                write!(f, "{}: {}", ContentLanguage::NAME, h)
            }
            Header::ContentLength(h) => {
                write!(f, "{}: {}", ContentLength::NAME, h)
            }
            Header::ContentType(h) => {
                write!(f, "{}: {}", ContentType::NAME, h)
            }
            Header::CSeq(h) => write!(f, "{}: {}", CSeq::NAME, h),
            Header::Date(h) => write!(f, "{}: {}", Date::NAME, h),
            Header::ErrorInfo(h) => write!(f, "{}: {}", ErrorInfo::NAME, h),
            Header::Expires(h) => write!(f, "{}: {}", Expires::NAME, h),
            Header::From(h) => write!(f, "{}: {}", From::NAME, h),
            Header::InReplyTo(h) => write!(f, "{}: {}", InReplyTo::NAME, h),
            Header::MaxForwards(h) => {
                write!(f, "{}: {}", MaxForwards::NAME, h)
            }
            Header::MinExpires(h) => {
                write!(f, "{}: {}", MinExpires::NAME, h)
            }
            Header::MimeVersion(h) => {
                write!(f, "{}: {}", MimeVersion::NAME, h)
            }
            Header::Organization(h) => {
                write!(f, "{}: {}", Organization::NAME, h)
            }
            Header::Priority(h) => write!(f, "{}: {}", Priority::NAME, h),
            Header::ProxyAuthenticate(h) => {
                write!(f, "{}: {}", ProxyAuthenticate::NAME, h)
            }
            Header::ProxyAuthorization(h) => {
                write!(f, "{}: {}", ProxyAuthorization::NAME, h)
            }
            Header::ProxyRequire(h) => {
                write!(f, "{}: {}", ProxyRequire::NAME, h)
            }
            Header::RetryAfter(h) => {
                write!(f, "{}: {}", RetryAfter::NAME, h)
            }
            Header::Route(h) => write!(f, "{}: {}", Route::NAME, h),
            Header::RecordRoute(h) => {
                write!(f, "{}: {}", RecordRoute::NAME, h)
            }
            Header::ReplyTo(h) => write!(f, "{}: {}", ReplyTo::NAME, h),
            Header::Require(h) => write!(f, "{}: {}", Require::NAME, h),
            Header::Server(h) => write!(f, "{}: {}", Server::NAME, h),
            Header::Subject(h) => write!(f, "{}: {}", Subject::NAME, h),
            Header::Supported(h) => write!(f, "{}: {}", Supported::NAME, h),
            Header::Timestamp(h) => write!(f, "{}: {}", Timestamp::NAME, h),
            Header::To(h) => write!(f, "{}: {}", To::NAME, h),
            Header::Unsupported(h) => {
                write!(f, "{}: {}", Unsupported::NAME, h)
            }
            Header::UserAgent(h) => write!(f, "{}: {}", UserAgent::NAME, h),
            Header::Warning(h) => write!(f, "{}: {}", Warning::NAME, h),
            Header::WWWAuthenticate(h) => {
                write!(f, "{}: {}", WWWAuthenticate::NAME, h)
            }
            Header::Other { name, value } => {
                write!(f, "{}: {}", name, value)
            }
        }?;
        write!(f, "\r\n")
    }
}

impl<'a> Header<'a> {
    /// Parses the header value as a string slice using the provided `reader`.
    pub fn parse_header_value_as_str(
        reader: &mut Reader<'a>,
    ) -> Result<&'a str> {
        let str = reader::until_newline!(reader);

        Ok(str::from_utf8(str)?)
    }

    /// Parses a SIP `Header` from a byte slice.
    ///
    /// # Examples
    /// ```
    /// # use sip::headers::{Header, ContentLength};
    /// let c_len = Header::ContentLength(ContentLength::new(10));
    ///
    /// assert_eq!(Header::from_bytes(b"Content-Length: 10"), Ok(c_len));
    /// ```
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let reader = &mut reader;

        let header_name = parse_token(reader)?;

        space!(reader);
        reader.must_read(b':')?;
        space!(reader);

        match header_name {
            Accept::NAME => Ok(Header::Accept(Accept::parse(reader)?)),
            AcceptEncoding::NAME => {
                Ok(Header::AcceptEncoding(AcceptEncoding::parse(reader)?))
            }
            AcceptLanguage::NAME => {
                Ok(Header::AcceptLanguage(AcceptLanguage::parse(reader)?))
            }
            AlertInfo::NAME => Ok(Header::AlertInfo(AlertInfo::parse(reader)?)),
            Allow::NAME => Ok(Header::Allow(Allow::parse(reader)?)),
            AuthenticationInfo::NAME => Ok(Header::AuthenticationInfo(
                AuthenticationInfo::parse(reader)?,
            )),
            Authorization::NAME => {
                Ok(Header::Authorization(Authorization::parse(reader)?))
            }
            CallId::NAME => Ok(Header::CallId(CallId::parse(reader)?)),
            CallInfo::NAME => Ok(Header::CallInfo(CallInfo::parse(reader)?)),
            Contact::NAME => Ok(Header::Contact(Contact::parse(reader)?)),
            ContentDisposition::NAME => Ok(Header::ContentDisposition(
                ContentDisposition::parse(reader)?,
            )),
            ContentEncoding::NAME => {
                Ok(Header::ContentEncoding(ContentEncoding::parse(reader)?))
            }
            ContentLanguage::NAME => {
                Ok(Header::ContentLanguage(ContentLanguage::parse(reader)?))
            }
            ContentLength::NAME => {
                Ok(Header::ContentLength(ContentLength::parse(reader)?))
            }
            ContentType::NAME => {
                Ok(Header::ContentType(ContentType::parse(reader)?))
            }
            CSeq::NAME => Ok(Header::CSeq(CSeq::parse(reader)?)),
            Date::NAME => Ok(Header::Date(Date::parse(reader)?)),
            ErrorInfo::NAME => Ok(Header::ErrorInfo(ErrorInfo::parse(reader)?)),
            Expires::NAME => Ok(Header::Expires(Expires::parse(reader)?)),
            From::NAME => Ok(Header::From(From::parse(reader)?)),
            InReplyTo::NAME => Ok(Header::InReplyTo(InReplyTo::parse(reader)?)),
            MaxForwards::NAME => {
                Ok(Header::MaxForwards(MaxForwards::parse(reader)?))
            }
            MinExpires::NAME => {
                Ok(Header::MinExpires(MinExpires::parse(reader)?))
            }
            MimeVersion::NAME => {
                Ok(Header::MimeVersion(MimeVersion::parse(reader)?))
            }
            Organization::NAME => {
                Ok(Header::Organization(Organization::parse(reader)?))
            }
            Priority::NAME => Ok(Header::Priority(Priority::parse(reader)?)),
            ProxyAuthenticate::NAME => {
                Ok(Header::ProxyAuthenticate(ProxyAuthenticate::parse(reader)?))
            }
            ProxyAuthorization::NAME => Ok(Header::ProxyAuthorization(
                ProxyAuthorization::parse(reader)?,
            )),
            ProxyRequire::NAME => {
                Ok(Header::ProxyRequire(ProxyRequire::parse(reader)?))
            }
            RetryAfter::NAME => {
                Ok(Header::RetryAfter(RetryAfter::parse(reader)?))
            }
            Route::NAME => Ok(Header::Route(Route::parse(reader)?)),
            RecordRoute::NAME => {
                Ok(Header::RecordRoute(RecordRoute::parse(reader)?))
            }
            ReplyTo::NAME => Ok(Header::ReplyTo(ReplyTo::parse(reader)?)),
            Require::NAME => Ok(Header::Require(Require::parse(reader)?)),
            Server::NAME => Ok(Header::Server(Server::parse(reader)?)),
            Subject::NAME => Ok(Header::Subject(Subject::parse(reader)?)),
            Supported::NAME => Ok(Header::Supported(Supported::parse(reader)?)),
            Timestamp::NAME => Ok(Header::Timestamp(Timestamp::parse(reader)?)),
            To::NAME => Ok(Header::To(To::parse(reader)?)),
            Unsupported::NAME => {
                Ok(Header::Unsupported(Unsupported::parse(reader)?))
            }
            UserAgent::NAME => Ok(Header::UserAgent(UserAgent::parse(reader)?)),
            Via::NAME => Ok(Header::Via(Via::parse(reader)?)),
            Warning::NAME => Ok(Header::Warning(Warning::parse(reader)?)),
            WWWAuthenticate::NAME => {
                Ok(Header::WWWAuthenticate(WWWAuthenticate::parse(reader)?))
            }
            _ => Ok(Header::Other {
                name: header_name,
                value: Self::parse_header_value_as_str(reader)?,
            }),
        }
    }
}

impl<'a> core::convert::From<Vec<Header<'a>>> for Headers<'a> {
    fn from(headers: Vec<Header<'a>>) -> Self {
        Self(headers)
    }
}

/// A set of SIP Headers.
///
/// A wrapper over Vec<[`Header`]> that contains the header list.
///
/// # Examples
/// ```
/// # use sip::headers::Headers;
/// # use sip::headers::Header;
/// # use sip::headers::ContentLength;
/// let mut headers = Headers::new();
/// headers.push(Header::ContentLength(ContentLength::new(10)));
///
/// assert_eq!(headers.len(), 1);
///
/// ```
#[derive(Debug)]
pub struct Headers<'a>(Vec<Header<'a>>);

impl<'a> Headers<'a> {
    /// Create a new empty collection of headers.
    ///
    /// # Examples
    /// ```
    /// # use sip::headers::Headers;
    /// let mut headers = Headers::new();
    /// ```
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Applies function to the headers and return the first no-none result.
    ///
    /// # Examples
    /// ```
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let headers = Headers::from(vec![
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

    /// Returns an iterator over headers.
    pub fn iter(&self) -> impl Iterator<Item = &Header<'a>> {
        self.0.iter()
    }

    /// Creates an iterator that both filters and maps an header.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let headers = Headers::from(vec![
    ///     Header::Expires(Expires::new(10))
    /// ]);
    /// let mut iter = headers.iter().filter_map(|h| match h {
    ///     Header::Expires(e) => Some(e),
    ///     _ => None
    /// });
    ///
    /// assert_eq!(iter.next(), Some(&Expires::new(10)));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn filter_map<'b, T: 'a, F>(
        &'b self,
        f: F,
    ) -> FilterMap<impl Iterator<Item = &Header<'a>>, F>
    where
        F: FnMut(&'b Header) -> Option<&'a T>,
    {
        self.0.iter().filter_map(f)
    }

    /// Creates an iterator which uses a closure to determine if an header should be yielded.
    ///
    /// # Examples
    ///
    /// ```
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let headers = Headers::from(vec![
    ///     Header::Expires(Expires::new(10))
    /// ]);
    ///
    /// let mut iter = headers.iter().filter(|h| matches!(h, Header::Expires(_)));
    ///
    /// assert_eq!(iter.next(), Some(&Header::Expires(Expires::new(10))));
    /// assert_eq!(iter.next(), None);
    /// ```
    pub fn filter<F>(
        &self,
        f: F,
    ) -> Filter<impl Iterator<Item = &Header<'a>>, F>
    where
        F: FnMut(&&Header) -> bool,
    {
        self.0.iter().filter(f)
    }

    /// Searches for an header that satisfies a predicate.
    /// # Examples
    ///
    /// ```
    /// # use sip::headers::Headers;
    /// # use sip::headers::Header;
    /// # use sip::headers::Expires;
    /// let headers = Headers::from(vec![
    ///     Header::Expires(Expires::new(10))
    /// ]);
    ///
    /// let header = headers.iter().find(|h| matches!(h, Header::Expires(_)));
    ///
    /// assert_eq!(header, Some(&Header::Expires(Expires::new(10))));
    /// ```
    pub fn find<F>(&self, f: F) -> Option<&Header>
    where
        F: FnMut(&&Header) -> bool,
    {
        self.0.iter().find(f)
    }
    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity exceeds `isize::MAX` bytes.
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }

    /// Push an new header.
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

    /// Returns the number of headers in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Get an reference to an header at the index specified.
    pub fn get(&self, index: usize) -> Option<&Header> {
        self.0.get(index)
    }
}

impl fmt::Display for Headers<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for hdr in self.iter() {
            write!(f, "{hdr}")?;
        }
        Ok(())
    }
}

impl Default for Headers<'_> {
    fn default() -> Self {
        Self::new()
    }
}
