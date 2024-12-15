//! SIP Message types
//!
//! The module provide the [`SipMessage`] enum that can be
//! an [`SipMessage::Request`] or [`SipMessage::Response`] and represents an sip message.

pub(crate) use request::*;
pub(crate) use response::*;

mod request;
mod response;

use core::fmt;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use crate::headers::{Header, Headers};

#[derive(Debug)]
pub enum SipMessage<'a> {
    Request(SipRequest<'a>),
    Response(SipResponse<'a>),
}

impl<'a> SipMessage<'a> {
    pub fn request(&self) -> Option<&SipRequest> {
        if let SipMessage::Request(req) = self {
            Some(req)
        } else {
            None
        }
    }
    pub fn headers(&self) -> &Headers<'a> {
        match self {
            SipMessage::Request(req) => &req.headers,
            SipMessage::Response(res) => &res.headers,
        }
    }
    pub fn headers_mut(&mut self) -> &mut Headers<'a> {
        match self {
            SipMessage::Request(req) => &mut req.headers,
            SipMessage::Response(res) => &mut res.headers,
        }
    }

    pub fn set_body(&mut self, body: Option<&'a [u8]>) {
        match self {
            SipMessage::Request(req) => req.body = body,
            SipMessage::Response(res) => res.body = body,
        }
    }

    pub fn push_header(&mut self, header: Header<'a>) {
        self.headers_mut().push(header);
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SipUri<'a> {
    Uri(Uri<'a>),
    NameAddr(NameAddr<'a>),
}

impl<'a> SipUri<'a> {
    pub fn uri(&self) -> Option<&Uri> {
        if let SipUri::Uri(uri) = self {
            Some(uri)
        } else {
            None
        }
    }

    pub fn scheme(&self) -> Scheme {
        match self {
            SipUri::Uri(uri) => uri.scheme,
            SipUri::NameAddr(name_addr) => name_addr.uri.scheme,
        }
    }

    pub fn host_port(&self) -> &HostPort {
        match self {
            SipUri::Uri(uri) => &uri.host_port,
            SipUri::NameAddr(name_addr) => &name_addr.uri.host_port,
        }
    }
    pub fn transport_param(&self) -> Option<TransportProtocol> {
        match self {
            SipUri::Uri(uri) => uri.transport_param,
            SipUri::NameAddr(name_addr) => name_addr.uri.transport_param,
        }
    }

    pub fn name_addr(&self) -> Option<&NameAddr> {
        if let SipUri::NameAddr(addr) = self {
            Some(addr)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Copy)]
pub enum Scheme {
    #[default]
    Sip,
    Sips,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    user: &'a str,
    pass: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    pub fn new(user: &'a str, pass: Option<&'a str>) -> Self {
        Self { user, pass}
    }
    pub fn get_user(&self) -> &'a str {
        self.user
    }
    pub fn get_pass(&self) -> Option<&'a str> {
        self.pass
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Host<'a> {
    DomainName(&'a str),
    IpAddr(IpAddr),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct HostPort<'a> {
    pub host: Host<'a>,
    pub port: Option<u16>,
}

impl<'a> HostPort<'a> {
    pub fn ip_addr(&self) -> Option<IpAddr> {
        match self.host {
            Host::DomainName(_) => None,
            Host::IpAddr(ip_addr) => Some(ip_addr),
        }
    }
}

impl<'a> From<Host<'a>> for HostPort<'a> {
    fn from(host: Host<'a>) -> Self {
        Self { host, port: None }
    }
}


impl Default for HostPort<'_> {
    fn default() -> Self {
        Self {
            host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            port: Some(5060),
        }
    }
}

impl<'a> HostPort<'a> {
    pub fn new(host: Host<'a>, port: Option<u16>) -> Self {
        Self { host, port }
    }

    pub fn is_domain(&self) -> bool {
        if let Host::DomainName(_) = self.host {
            true
        } else {
            false
        }
    }
    pub fn host_as_str(&self) -> String {
        match self.host {
            Host::DomainName(host) => host.to_string(),
            Host::IpAddr(host) => host.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Uri<'a> {
    pub scheme: Scheme,
    pub user: Option<UserInfo<'a>>,
    pub host_port: HostPort<'a>,
    pub user_param: Option<&'a str>,
    pub method_param: Option<&'a str>,
    pub transport_param: Option<TransportProtocol>,
    pub ttl_param: Option<&'a str>,
    pub lr_param: Option<&'a str>,
    pub maddr_param: Option<&'a str>,
    pub params: Option<Params<'a>>,
    pub hdr_params: Option<Params<'a>>,
}

impl<'a> Uri<'a> {
    pub fn without_params(
        scheme: Scheme,
        user: Option<UserInfo<'a>>,
        host_port: HostPort<'a>,
    ) -> Self {
        Uri {
            scheme,
            user,
            host_port,
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct UriBuilder<'a> {
    uri: Uri<'a>,
}

impl<'a> UriBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.uri.scheme = scheme;
        self
    }

    pub fn user(mut self, user: UserInfo<'a>) -> Self {
        self.uri.user = Some(user);
        self
    }

    pub fn host(mut self, host_port: HostPort<'a>) -> Self {
        self.uri.host_port = host_port;
        self
    }

    pub fn user_param(mut self, user_param: &'a str) -> Self {
        self.uri.user_param = Some(user_param);
        self
    }

    pub fn method_param(mut self, method_param: &'a str) -> Self {
        self.uri.method_param = Some(method_param);
        self
    }
    pub fn transport_param(mut self, transport_param: TransportProtocol) -> Self {
        self.uri.transport_param = Some(transport_param);
        self
    }

    pub fn ttl_param(mut self, ttl_param: &'a str) -> Self {
        self.uri.ttl_param = Some(ttl_param);
        self
    }
    pub fn lr_param(mut self, lr_param: &'a str) -> Self {
        self.uri.lr_param = Some(lr_param);
        self
    }

    pub fn maddr_param(mut self, maddr_param: &'a str) -> Self {
        self.uri.maddr_param = Some(maddr_param);
        self
    }

    pub fn params(mut self, params: Params<'a>) -> Self {
        self.uri.params = Some(params);
        self
    }

    pub fn get(self) -> Uri<'a> {
        self.uri
    }
}

// SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
#[derive(Debug, PartialEq, Eq)]
pub struct NameAddr<'a> {
    pub display: Option<&'a str>,
    pub uri: Uri<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenericUri<'a> {
    pub(crate) scheme: &'a str,
    pub(crate) content: &'a str,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Params<'a> {
    pub(crate) inner: HashMap<&'a str, &'a str>,
}

impl<'a> From<HashMap<&'a str, &'a str>> for Params<'a> {
    fn from(value: HashMap<&'a str, &'a str>) -> Self {
        Self { inner: value }
    }
}

impl<'a> Params<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(&mut self, k: &'a str, v: &'a str) -> Option<&str> {
        self.inner.insert(k, v)
    }
    pub fn get(&self, k: &'a str) -> Option<&&str> {
        self.inner.get(k)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SipMethod {
    Invite,
    Ack,
    Bye,
    Cancel,
    Register,
    Options,
    Info,
    Notify,
    Subscribe,
    Update,
    Refer,
    Prack,
    Message,
    Publish,
    Unknow
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransportProtocol {
    #[default] UDP,
    TCP,
    TLS,
    SCTP,
    Unknow,
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportProtocol::UDP => write!(f,"{}", TP_UDP),
            TransportProtocol::TCP => write!(f, "{}", TP_TCP),
            TransportProtocol::TLS => write!(f, "{}", TP_TLS),
            TransportProtocol::SCTP => write!(f, "{}", TP_SCTP),
            TransportProtocol::Unknow => write!(f, "{}", TP_UNKNOW),
        }
    }
}


impl TransportProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransportProtocol::UDP => TP_UDP,
            TransportProtocol::TCP => TP_TCP,
            TransportProtocol::TLS => TP_TLS,
            TransportProtocol::SCTP => TP_SCTP,
            TransportProtocol::Unknow => TP_UNKNOW,
        }
    }
}

const TP_UDP: &str = "UDP";
const TP_TCP: &str = "TCP";
const TP_TLS: &str = "TLS";
const TP_SCTP: &str = "SCTP";
const TP_UNKNOW: &str = "UNKNOW-TP";

const B_UDP: &[u8] = TP_UDP.as_bytes();
const B_TCP: &[u8] = TP_TCP.as_bytes();
const B_TLS: &[u8] = TP_TLS.as_bytes();
const B_SCTP: &[u8] = TP_SCTP.as_bytes();

impl From<&str> for TransportProtocol {
    fn from(value: &str) -> Self {
        match value {
            TP_UDP => TransportProtocol::UDP,
            TP_TCP => TransportProtocol::TCP,
            TP_TLS => TransportProtocol::TLS,
            TP_SCTP => TransportProtocol::SCTP,
            _ => TransportProtocol::Unknow,
        }
    }
}

impl From<&[u8]> for TransportProtocol {
    fn from(value: &[u8]) -> Self {
        match value {
            B_UDP => TransportProtocol::UDP,
            B_TCP => TransportProtocol::TCP,
            B_TLS => TransportProtocol::TLS,
            B_SCTP => TransportProtocol::SCTP,
            _ => TransportProtocol::Unknow,
        }
    }
}

const SIP_INVITE: &[u8] = b"INVITE";
const SIP_CANCEL: &[u8] = b"CANCEL";
const SIP_ACK: &[u8] = b"ACK";
const SIP_BYE: &[u8] = b"BYE";
const SIP_REGISTER: &[u8] = b"REGISTER";
const SIP_OPTIONS: &[u8] = b"OPTIONS";
const SIP_INFO: &[u8] = b"INFO";
const SIP_NOTIFY: &[u8] = b"NOTIFY";
const SIP_SUBSCRIBE: &[u8] = b"SUBSCRIBE";
const SIP_UPDATE: &[u8] = b"UPDATE";
const SIP_REFER: &[u8] = b"REFER";
const SIP_PRACK: &[u8] = b"PRACK";
const SIP_MESSAGE: &[u8] = b"MESSAGE";
const SIP_PUBLISH: &[u8] = b"PUBLISH";

impl From<&[u8]> for SipMethod {
    fn from(value: &[u8]) -> Self {
        match value {
            SIP_INVITE => SipMethod::Invite,
            SIP_CANCEL => SipMethod::Cancel,
            SIP_ACK => SipMethod::Ack,
            SIP_BYE => SipMethod::Bye,
            SIP_REGISTER => SipMethod::Register,
            SIP_OPTIONS => SipMethod::Options,
            SIP_INFO => SipMethod::Info,
            SIP_NOTIFY => SipMethod::Notify,
            SIP_SUBSCRIBE => SipMethod::Subscribe,
            SIP_UPDATE => SipMethod::Update,
            SIP_REFER => SipMethod::Refer,
            SIP_PRACK => SipMethod::Prack,
            SIP_MESSAGE => SipMethod::Message,
            SIP_PUBLISH => SipMethod::Publish,
            _ => SipMethod::Unknow,
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
    CallOrTransactionDoesNotExist = 481,
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

    ServerInternalError = 500,
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
//@TODO: complete status codes
impl SipStatusCode {
    pub fn reason_phrase(&self) -> &str {
        match &self {
            // 1xx — Provisional Responses
            SipStatusCode::Trying => "Trying",
            SipStatusCode::Ringing => "Ringing",
            SipStatusCode::CallIsBeingForwarded => "Call Is Being Forwarded",
            SipStatusCode::Queued => "Queued",
            SipStatusCode::SessionProgress => "Session Progress",

            // 2xx — Successful Responses
            SipStatusCode::Ok => "OK",
            SipStatusCode::Accepted => "Accepted",
            SipStatusCode::NoNotification => "No Notification",

            // 3xx — Redirection Responses
            SipStatusCode::MultipleChoices => "Multiple Choices",
            SipStatusCode::MovedPermanently => "Moved Permanently",
            SipStatusCode::MovedTemporarily => "Moved Temporarily",
            SipStatusCode::UseProxy => "Use Proxy",
            SipStatusCode::AlternativeService => "Alternative Service",

            // 4xx — Client Failure Responses
            SipStatusCode::BadRequest => "Bad Request",
            SipStatusCode::Unauthorized => "Unauthorized",
            SipStatusCode::PaymentRequired => "Payment Required",
            SipStatusCode::Forbidden => "Forbidden",
            SipStatusCode::NotFound => "Not Found",
            SipStatusCode::MethodNotAllowed => "Method Not Allowed",
            SipStatusCode::NotAcceptable => "Not Acceptable",
            SipStatusCode::ProxyAuthenticationRequired => {
                "Proxy Authentication Required"
            }
            SipStatusCode::RequestTimeout => "Request Timeout",
            SipStatusCode::Gone => "Gone",
            SipStatusCode::RequestEntityTooLarge => "Request Entity Too Large",
            SipStatusCode::RequestUriTooLong => "Request-URI Too Long",
            SipStatusCode::UnsupportedMediaType => "Unsupported Media Type",
            SipStatusCode::UnsupportedUriScheme => "Unsupported URI Scheme",
            SipStatusCode::BadExtension => "Bad Extension",
            SipStatusCode::ExtensionRequired => "Extension Required",
            SipStatusCode::IntervalTooBrief => "Interval Too Brief",
            SipStatusCode::TemporarilyUnavailable => "Temporarily Unavailable",
            SipStatusCode::CallOrTransactionDoesNotExist => {
                "Call/Transaction Does Not Exist"
            }
            SipStatusCode::LoopDetected => "Loop Detected",
            SipStatusCode::TooManyHops => "Too Many Hops",
            SipStatusCode::AddressIncomplete => "Address Incomplete",
            SipStatusCode::Ambiguous => "Ambiguous",
            SipStatusCode::BusyHere => "Busy Here",
            SipStatusCode::RequestTerminated => "Request Terminated",
            SipStatusCode::NotAcceptableHere => "Not Acceptable Here",
            SipStatusCode::RequestPending => "Request Pending",
            SipStatusCode::Undecipherable => "Undecipherable",

            // 5xx — Server Failure Responses
            SipStatusCode::ServerInternalError => "Server Internal Error",
            SipStatusCode::NotImplemented => "Not Implemented",
            SipStatusCode::BadGateway => "Bad Gateway",
            SipStatusCode::ServiceUnavailable => "Service Unavailable",
            SipStatusCode::ServerTimeout => "Server Time-out",
            SipStatusCode::VersionNotSupported => "Version Not Supported",
            SipStatusCode::MessageTooLarge => "Message Too Large",

            // 6xx — Global Failure Responses
            SipStatusCode::BusyEverywhere => "Busy Everywhere",
            SipStatusCode::Decline => "Decline",
            SipStatusCode::DoesNotExistAnywhere => "Does Not Exist Anywhere",
            SipStatusCode::NotAcceptableAnywhere => "Not Acceptable",

            // Unknown or custom status
            _ => "Unknown",
        }
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
            b"481" => SipStatusCode::CallOrTransactionDoesNotExist,
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
            b"500" => SipStatusCode::ServerInternalError,
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
