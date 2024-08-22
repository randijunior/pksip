pub mod to;
pub mod from;
pub mod via;

pub use to::To;
pub use via::Via;
pub use from::From;

use crate::{byte_reader::ByteReader, parser::Result};


pub(crate) trait SipHeaderParser<'a>: Sized {
    const NAME: &'a [u8];
    const SHORT_NAME: Option<&'a [u8]> = None;

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self>;

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME) || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }
}

pub struct SipHeaders<'a> {
    pub(crate) hdrs: Vec<Header<'a>>,
}

impl<'a> SipHeaders<'a> {
    pub fn new() -> Self {
        Self { hdrs: vec![] }
    }
    pub fn push_header(&mut self, hdr: Header<'a>) {
        self.hdrs.push(hdr);
    }
}


pub enum Header<'a> {
    Accept,
    AcceptEncoding,
    AcceptLanguage,
    AlertInfo,
    Allow,
    AuthenticationInfo,
    Authorization,
    CallID,
    CallInfo,
    Contact,
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentType,
    CSeq,
    Date,
    ErrorInfo,
    Expires,
    From(From<'a>),
    InReplyTo,
    MaxForwards,
    MimeVersion,
    MinExpires,
    Organization,
    Priority,
    ProxyAuthenticate,
    ProxyAuthorization,
    ProxyRequire,
    RecordRoute,
    ReplyTo,
    Require,
    RetryAfter,
    Route,
    Server,
    Subject,
    Supported,
    Timestamp,
    To(To<'a>),
    Unsupported,
    UserAgent,
    Via(Via<'a>),
    Warning,
    WWWAuthenticate,
    Other,
}

