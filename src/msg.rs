use crate::{
   headers::SipHeaders, uri::Uri
};

use std::str;

pub struct SipRequest<'a> {
    req_line: RequestLine<'a>,
    headers: SipHeaders<'a>,
    body: &'a [u8],
}


pub struct SipResponse<'a> {
    req_line: StatusLine<'a>,
    headers: SipHeaders<'a>,
    body: &'a [u8],
}

/// This struct represent SIP Message
pub enum SipMsg<'a> {
    Request(SipRequest<'a>),
    Response(SipResponse<'a>),
}

/// This struct represent SIP status line
#[derive(Debug, PartialEq, Eq)]
pub struct StatusLine<'sl> {
    // Status Code
    pub(crate) status_code: SipStatusCode,
    // Reason String
    pub(crate) reason_phrase: &'sl str,
}

impl<'sl> StatusLine<'sl> {
    pub fn new(st: SipStatusCode, rp: &'sl str) -> Self {
        StatusLine {
            status_code: st,
            reason_phrase: rp,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RequestLine<'a> {
    pub(crate) method: SipMethod<'a>,
    pub(crate) uri: Uri<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SipMethod<'a> {
    Invite,
    Ack,
    Bye,
    Cancel,
    Register,
    Options,
    Other(&'a [u8]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    UDP,
    TCP,
    TLS,
    SCTP,
    Unknow,
}

const TRANSPORT_UDP: &[u8] = "UDP".as_bytes();
const TRANSPORT_TCP: &[u8] = "TCP".as_bytes();
const TRANSPORT_TLS: &[u8] = "TLS".as_bytes();
const TRANSPORT_SCTP: &[u8] = "SCTP".as_bytes();

impl From<&[u8]> for Transport {
    fn from(value: &[u8]) -> Self {
        match value {
            TRANSPORT_UDP => Transport::UDP,
            TRANSPORT_TCP => Transport::TCP,
            TRANSPORT_TLS => Transport::TLS,
            TRANSPORT_SCTP => Transport::SCTP,
            _ => Transport::Unknow,
        }
    }
}

const SIP_INVITE: &[u8] = "INVITE".as_bytes();
const SIP_CANCEL: &[u8] = "CANCEL".as_bytes();
const SIP_ACK: &[u8] = "ACK".as_bytes();
const SIP_BYE: &[u8] = "BYE".as_bytes();
const SIP_REGISTER: &[u8] = "REGISTER".as_bytes();
const SIP_OPTIONS: &[u8] = "OPTIONS".as_bytes();

impl<'a> From<&'a [u8]> for SipMethod<'a> {
    fn from(value: &'a [u8]) -> Self {
        match value {
            SIP_INVITE => SipMethod::Invite,
            SIP_CANCEL => SipMethod::Cancel,
            SIP_ACK => SipMethod::Ack,
            SIP_BYE => SipMethod::Bye,
            SIP_REGISTER => SipMethod::Register,
            SIP_OPTIONS => SipMethod::Options,
            _ => SipMethod::Other(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SipStatusCode {
    Trying = 100,
    Ringing = 180,
    CallIsBeingForwarded = 181,
    Queued = 182,
    SessionProgress = 183,
    EarlyDialogTerminated = 199,

    Ok = 200,
    Accepted = 202,
    NoNotification = 204,

    MultipleChoices = 300,
    MovedPermanently = 301,
    MovedTemporarily = 302,
    UseProxy = 305,
    AlternativeService = 380,

    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    ConditionalRequestFailed = 412,
    RequestEntityTooLarge = 413,
    RequestUriTooLong = 414,
    UnsupportedMediaType = 415,
    UnsupportedUriScheme = 416,
    UnknownResourcePriority = 417,
    BadExtension = 420,
    ExtensionRequired = 421,
    SessionTimerTooSmall = 422,
    IntervalTooBrief = 423,
    BadLocationInformation = 424,
    UseIdentityHeader = 428,
    ProvideReferrerHeader = 429,
    FlowFailed = 430,
    AnonimityDisallowed = 433,
    BadIdentityInfo = 436,
    UnsupportedCertificate = 437,
    InvalidIdentityHeader = 438,
    FirstHopLacksOutboundSupport = 439,
    MaxBreadthExceeded = 440,
    BadInfoPackage = 469,
    ConsentNeeded = 470,
    TemporarilyUnavailable = 480,
    CallTsxDoesNotExist = 481,
    LoopDetected = 482,
    TooManyHops = 483,
    AddressIncomplete = 484,
    Ambiguous = 485,
    BusyHere = 486,
    RequestTerminated = 487,
    NotAcceptableHere = 488,
    BadEvent = 489,
    RequestUpdated = 490,
    RequestPending = 491,
    Undecipherable = 493,
    SecurityAgreementNeeded = 494,

    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    ServerTimeout = 504,
    VersionNotSupported = 505,
    MessageTooLarge = 513,
    PushNotificationServiceNotSupported = 555,
    PreconditionFailure = 580,

    BusyEverywhere = 600,
    Decline = 603,
    DoesNotExistAnywhere = 604,
    NotAcceptableAnywhere = 606,
    Unwanted = 607,
    Rejected = 608,
    Unknow,
}

// https://en.wikipedia.org/wiki/List_of_SIP_response_codes
impl SipStatusCode {
    pub fn reason_phrase(&self) -> &str {
        match &self {
            // 1xx — Provisional Responses
            SipStatusCode::Trying => "Trying",
            SipStatusCode::Ringing => "Ringing",
            SipStatusCode::CallIsBeingForwarded => "Call is Being Forwarded",
            SipStatusCode::Queued => "Queued",
            SipStatusCode::SessionProgress => "Session Progress",

            // 2xx — Successful Responses
            SipStatusCode::Ok => "OK",
            SipStatusCode::Accepted => "Accepted",
            SipStatusCode::NoNotification => "No Notification",

            // 3xx — Redirection Responses
            SipStatusCode::MultipleChoices => "Multiple Choices",

            // 4xx - Client Failure Responses
            SipStatusCode::NotFound => "Not Found",
            _ => "Unknow",
        }
    }

    pub fn reason_phrase_bytes(&self) -> &[u8] {
        self.reason_phrase().as_bytes()
    }
}

impl From<&[u8]> for SipStatusCode {
    fn from(value: &[u8]) -> Self {
        match value {
            b"100" => SipStatusCode::Trying,
            b"180" => SipStatusCode::Ringing,
            b"181" => SipStatusCode::CallIsBeingForwarded,
            b"182" => SipStatusCode::Queued,
            b"183" => SipStatusCode::SessionProgress,
            b"199" => SipStatusCode::EarlyDialogTerminated,
            b"200" => SipStatusCode::Ok,
            b"202" => SipStatusCode::Accepted,
            b"204" => SipStatusCode::NoNotification,
            b"300" => SipStatusCode::MultipleChoices,
            b"301" => SipStatusCode::MovedPermanently,
            b"302" => SipStatusCode::MovedTemporarily,
            b"305" => SipStatusCode::UseProxy,
            b"380" => SipStatusCode::AlternativeService,
            b"400" => SipStatusCode::BadRequest,
            b"401" => SipStatusCode::Unauthorized,
            b"402" => SipStatusCode::PaymentRequired,
            b"403" => SipStatusCode::Forbidden,
            b"404" => SipStatusCode::NotFound,
            b"405" => SipStatusCode::MethodNotAllowed,
            b"406" => SipStatusCode::NotAcceptable,
            b"407" => SipStatusCode::ProxyAuthenticationRequired,
            b"408" => SipStatusCode::RequestTimeout,
            b"409" => SipStatusCode::Conflict,
            b"410" => SipStatusCode::Gone,
            b"411" => SipStatusCode::LengthRequired,
            b"412" => SipStatusCode::ConditionalRequestFailed,
            b"413" => SipStatusCode::RequestEntityTooLarge,
            b"414" => SipStatusCode::RequestUriTooLong,
            b"415" => SipStatusCode::UnsupportedMediaType,
            b"416" => SipStatusCode::UnsupportedUriScheme,
            b"417" => SipStatusCode::UnknownResourcePriority,
            b"420" => SipStatusCode::BadExtension,
            b"421" => SipStatusCode::ExtensionRequired,
            b"422" => SipStatusCode::SessionTimerTooSmall,
            b"423" => SipStatusCode::IntervalTooBrief,
            b"424" => SipStatusCode::BadLocationInformation,
            b"428" => SipStatusCode::UseIdentityHeader,
            b"429" => SipStatusCode::ProvideReferrerHeader,
            b"430" => SipStatusCode::FlowFailed,
            b"433" => SipStatusCode::AnonimityDisallowed,
            b"436" => SipStatusCode::BadIdentityInfo,
            b"437" => SipStatusCode::UnsupportedCertificate,
            b"438" => SipStatusCode::InvalidIdentityHeader,
            b"439" => SipStatusCode::FirstHopLacksOutboundSupport,
            b"440" => SipStatusCode::MaxBreadthExceeded,
            b"469" => SipStatusCode::BadInfoPackage,
            b"470" => SipStatusCode::ConsentNeeded,
            b"480" => SipStatusCode::TemporarilyUnavailable,
            b"481" => SipStatusCode::CallTsxDoesNotExist,
            b"482" => SipStatusCode::LoopDetected,
            b"483" => SipStatusCode::TooManyHops,
            b"484" => SipStatusCode::AddressIncomplete,
            b"485" => SipStatusCode::Ambiguous,
            b"486" => SipStatusCode::BusyHere,
            b"487" => SipStatusCode::RequestTerminated,
            b"488" => SipStatusCode::NotAcceptableHere,
            b"489" => SipStatusCode::BadEvent,
            b"490" => SipStatusCode::RequestUpdated,
            b"491" => SipStatusCode::RequestPending,
            b"493" => SipStatusCode::Undecipherable,
            b"494" => SipStatusCode::SecurityAgreementNeeded,
            b"500" => SipStatusCode::InternalServerError,
            b"501" => SipStatusCode::NotImplemented,
            b"502" => SipStatusCode::BadGateway,
            b"503" => SipStatusCode::ServiceUnavailable,
            b"504" => SipStatusCode::ServerTimeout,
            b"505" => SipStatusCode::VersionNotSupported,
            b"513" => SipStatusCode::MessageTooLarge,
            b"555" => SipStatusCode::PushNotificationServiceNotSupported,
            b"580" => SipStatusCode::PreconditionFailure,
            b"600" => SipStatusCode::BusyEverywhere,
            b"603" => SipStatusCode::Decline,
            b"604" => SipStatusCode::DoesNotExistAnywhere,
            b"606" => SipStatusCode::NotAcceptableAnywhere,
            b"607" => SipStatusCode::Unwanted,
            b"608" => SipStatusCode::Rejected,
            _ => SipStatusCode::Unknow,
        }
    }
}
