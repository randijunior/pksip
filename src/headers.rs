pub mod to;

use to::To;

pub struct SipHeaders<'a> {
    pub(crate) hdrs: Vec<Header<'a>>,
}

pub enum Header<'a> {
    Accept,
    AcceptEncodingUnimp,
    AcceptLanguageUnimp,
    AlertInfoUnimp,
    Allow,
    AuthenticationInfoUnimp,
    Authorization,
    CallId,
    CallInfoUnimp,
    Contact,
    ContentDispositionUnimp,
    ContentEncodingUnimp,
    ContentLanguageUnimp,
    ContentLength,
    ContentType,
    Cseq,
    DateUnimp,
    ErrorInfoUnimp,
    Expires,
    From,
    InReplyToUnimp,
    MaxForwards,
    MimeVersionUnimp,
    MinExpires,
    OrganizationUnimp,
    PriorityUnimp,
    ProxyAuthenticate,
    ProxyAuthorization,
    ProxyRequireUnimp,
    RecordRoute,
    ReplyToUnimp,
    Require,
    RetryAfter,
    Route,
    ServerUnimp,
    SubjectUnimp,
    Supported,
    TimestampUnimp,
    To(To<'a>),
    Unsupported,
    UserAgentUnimp,
    Via,
    WarningUnimp,
    WwwAuthenticate,
    Other,
}
