pub mod call_id;
pub mod contact;
pub mod from;
pub mod route;
pub mod to;
pub mod via;
pub mod accept;
pub mod max_fowards;
pub mod cseq;
pub mod expires;
pub mod allow;
pub mod accept_encoding;
pub mod accept_language;
pub mod alert_info;
pub mod auth_info;

use std::str;

use accept::Accept;
use accept_encoding::AcceptEncoding;
use accept_language::AcceptLanguage;
use alert_info::AlertInfo;
use allow::Allow;
use auth_info::AuthenticationInfo;
pub use call_id::CallId;
use contact::Contact;
use cseq::CSeq;
use expires::Expires;
pub use from::From;
use max_fowards::MaxForwards;
use route::Route;
pub use to::To;
pub use via::Via;

use crate::{
    byte_reader::ByteReader,
    macros::read_while,
    parser::{is_token, Result},
};

pub(crate) trait SipHeaderParser<'a>: Sized {
    const NAME: &'a [u8];
    const SHORT_NAME: Option<&'a [u8]> = None;

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self>;

    #[inline]
    fn match_name(name: &[u8]) -> bool {
        name.eq_ignore_ascii_case(Self::NAME)
            || Self::SHORT_NAME.is_some_and(|s_name| name == s_name)
    }

    fn parse_param(reader: &mut ByteReader<'a>) -> Result<(&'a str, Option<&'a str>)> {
        // take ';' character
        reader.next();

        let name = read_while!(reader, is_token);
        let name = unsafe { str::from_utf8_unchecked(name) };
        let value = if reader.peek() == Some(&b'=') {
            reader.next();
            let value = read_while!(reader, is_token);
            Some(unsafe { str::from_utf8_unchecked(value) })
        } else {
            None
        };

        Ok((name, value))
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
    Accept(Accept<'a>),
    AcceptEncoding(AcceptEncoding<'a>),
    AcceptLanguage(AcceptLanguage<'a>),
    AlertInfo(AlertInfo<'a>),
    Allow(Allow<'a>),
    AuthenticationInfo(AuthenticationInfo<'a>),
    Authorization,
    CallId(CallId<'a>),
    CallInfo,
    Contact(Contact<'a>),
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentType,
    CSeq(CSeq<'a>),
    Date,
    ErrorInfo,
    Expires(Expires),
    From(From<'a>),
    InReplyTo,
    MaxForwards(MaxForwards),
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
