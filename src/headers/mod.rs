mod call_id;
pub mod contact;
pub mod from;
pub mod route;
pub mod to;
pub mod via;

pub use call_id::CallId;
use contact::Contact;
pub use from::From;
use route::Route;
pub use to::To;
pub use via::Via;

use crate::{
    byte_reader::ByteReader,
    macros::read_while,
    parser::{is_token, Result},
};
use std::str;

pub(crate) trait SipHeaderParser<'a>: Sized {
    const NAME: &'a [u8];
    const SHORT_NAME: Option<&'a [u8]> = None;

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self>;

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME)
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }

    fn parse_param(reader: &mut ByteReader<'a>) -> Result<(&'a [u8], Option<&'a str>)> {
        // take ';' character
        reader.next();

        let name = read_while!(reader, is_token);
        let value = if reader.peek() == Some(&b'=') {
            reader.next();
            let value = read_while!(reader, is_token);
            Some(str::from_utf8(value)?)
        } else {
            None
        };

        Ok((name, value))
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

// Headers, as defined in RFC3261.
pub enum Header<'a> {
    Accept,
    AcceptEncoding,
    AcceptLanguage,
    AlertInfo,
    Allow,
    AuthenticationInfo,
    Authorization,
    CallId(CallId<'a>),
    CallInfo,
    Contact(Contact<'a>),
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
    Route(Route<'a>),
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
    Other { name: &'a str, value: &'a str },
}
