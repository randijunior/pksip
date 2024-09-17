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
pub mod route;
pub mod to;
pub mod via;
pub mod mime_version;
pub mod min_expires;
pub mod organization;
pub mod priority;

use std::str;

use accept::Accept;
use accept_encoding::AcceptEncoding;
use accept_language::AcceptLanguage;
use alert_info::AlertInfo;
use allow::Allow;
use authentication_info::AuthenticationInfo;
use authorization::Authorization;
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
use route::Route;
pub use to::To;
pub use via::Via;

use crate::{
    byte_reader::ByteReader,
    macros::read_while,
    parser::{is_token, Result},
};

pub(crate) fn parse_generic_param<'a>(
    reader: &mut ByteReader<'a>,
) -> Result<(&'a str, Option<&'a str>)> {
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

pub(crate) trait SipHeaderParser<'a>: Sized {
    const NAME: &'a [u8];
    const SHORT_NAME: Option<&'a [u8]> = None;

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self>;

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
    Generic { name: &'a str, value: &'a str },
}
