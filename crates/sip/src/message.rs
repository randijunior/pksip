//! SIP Message types
//!
//! The module provide the [`SipMessage`] enum that can be
//! an [`SipMessage::Request`] or [`SipMessage::Response`] and represents an sip message.

use itertools::Itertools;
use reader::Reader;
pub use request::*;
pub use response::*;

mod request;
mod response;

use core::fmt;
use std::collections::HashMap;
use std::fmt::Debug;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use crate::headers::{Header, Headers};
use crate::internal::ArcStr;
use crate::parser::SipParserError;

#[derive(Debug)]
pub enum SipMessage {
    Request(SipRequest),
    Response(SipResponse),
}

impl SipMessage {
    pub fn request(&self) -> Option<&SipRequest> {
        if let SipMessage::Request(req) = self {
            Some(req)
        } else {
            None
        }
    }
    pub fn headers(&self) -> &Headers {
        match self {
            SipMessage::Request(req) => &req.headers,
            SipMessage::Response(res) => &res.headers,
        }
    }

    pub fn body(&self) -> &Option<&[u8]> {
        // match self {
        //     SipMessage::Request(req) => req.body,
        //     SipMessage::Response(res) => &res.body,
        // }
        todo!()
    }

    pub fn headers_mut(&mut self) -> &mut Headers {
        match self {
            SipMessage::Request(req) => &mut req.headers,
            SipMessage::Response(res) => &mut res.headers,
        }
    }

    pub fn set_body(&mut self, body: Option<&[u8]>) {
        match self {
            SipMessage::Request(req) => req.body = body.map(|b| b.into()),
            SipMessage::Response(res) => res.body = body.map(|b| b.into()),
        }
    }

    pub fn push_header(&mut self, header: Header) {
        self.headers_mut().push(header);
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SipUri {
    Uri(Uri),
    NameAddr(NameAddr),
}

impl FromStr for SipUri {
    type Err = SipParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut reader = Reader::new(s.as_bytes());

        crate::parser::parse_sip_uri(&mut reader, true)
    }
}

impl fmt::Display for SipUri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SipUri::Uri(uri) => write!(f, "{}", uri),
            SipUri::NameAddr(name_addr) => write!(f, "{}", name_addr),
        }
    }
}

impl SipUri {
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserInfo {
    user: ArcStr,
    pass: Option<ArcStr>,
}

impl UserInfo {
    pub fn new(user: &str, pass: Option<&str>) -> Self {
        Self {
            user: user.into(),
            pass: pass.map(|s| s.into()),
        }
    }
    pub fn get_user(&self) -> &str {
        &*self.user
    }
    pub fn get_pass(&self) -> Option<&str> {
        self.pass.as_deref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Host {
    DomainName(ArcStr),
    IpAddr(IpAddr),
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Host::DomainName(domain) => write!(f, "{domain}"),
            Host::IpAddr(ip_addr) => write!(f, "{ip_addr}"),
        }
    }
}

impl Host {
    pub fn is_ip_addr(&self) -> bool {
        match self {
            Host::DomainName(_) => false,
            Host::IpAddr(_) => true,
        }
    }
    pub fn as_str(&self) -> String {
        match self {
            Host::DomainName(host) => String::from(host.as_ref() as &str),
            Host::IpAddr(host) => host.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct HostPort {
    pub host: Host,
    pub port: Option<u16>,
}

impl HostPort {
    pub fn ip_addr(&self) -> Option<IpAddr> {
        match self.host {
            Host::DomainName(_) => None,
            Host::IpAddr(ip_addr) => Some(ip_addr),
        }
    }
    pub fn is_ip_addr(&self) -> bool {
        self.ip_addr().is_some()
    }
}

impl fmt::Display for HostPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.host {
            Host::DomainName(domain) => write!(f, "{domain}")?,
            Host::IpAddr(ip_addr) => write!(f, "{ip_addr}")?,
        }
        if let Some(port) = self.port {
            write!(f, ":{port}")?;
        }
        Ok(())
    }
}

impl From<Host> for HostPort {
    fn from(host: Host) -> Self {
        Self { host, port: None }
    }
}

impl Default for HostPort {
    fn default() -> Self {
        Self {
            host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            port: Some(5060),
        }
    }
}

impl HostPort {
    pub fn new(host: Host, port: Option<u16>) -> Self {
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
        self.host.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Uri {
    pub scheme: Scheme,
    pub user: Option<UserInfo>,
    pub host_port: HostPort,
    pub user_param: Option<ArcStr>,
    pub method_param: Option<ArcStr>,
    pub transport_param: Option<TransportProtocol>,
    pub ttl_param: Option<ArcStr>,
    pub lr_param: Option<ArcStr>,
    pub maddr_param: Option<ArcStr>,
    pub params: Option<Params>,
    pub hdr_params: Option<Params>,
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.scheme {
            Scheme::Sip => write!(f, "sip")?,
            Scheme::Sips => write!(f, "sips")?,
        }
        write!(f, ":")?;

        if let Some(user) = &self.user {
            write!(f, "{}", user.get_user())?;
            if let Some(pass) = user.get_pass() {
                write!(f, ":{}", pass)?
            }
            write!(f, "@")?;
        }
        write!(f, "{}", self.host_port)?;

        if let Some(user_param) = &self.user_param {
            write!(f, ";user={}", user_param)?;
        }
        if let Some(m_param) = &self.method_param {
            write!(f, ";method={}", m_param)?;
        }
        if let Some(m_param) = &self.maddr_param {
            write!(f, ";maddr={}", m_param)?;
        }
        if let Some(transport_param) = &self.transport_param {
            write!(f, ";transport={}", transport_param)?;
        }
        if let Some(ttl_param) = &self.ttl_param {
            write!(f, ";ttl={}", ttl_param)?;
        }
        if let Some(lr_param) = &self.lr_param {
            write!(f, ";lr={}", lr_param)?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        if let Some(hdr_params) = &self.hdr_params {
            let formater = hdr_params.iter().format_with("&", |it, f| {
                f(&format_args!("{}={}", it.0, it.1))
            });
            write!(f, "?{}", formater)?;
        }

        Ok(())
    }
}

impl Uri {
    pub fn without_params(
        scheme: Scheme,
        user: Option<UserInfo>,
        host_port: HostPort,
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
pub struct UriBuilder {
    uri: Uri,
}

impl UriBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn scheme(mut self, scheme: Scheme) -> Self {
        self.uri.scheme = scheme;
        self
    }

    pub fn user(mut self, user: UserInfo) -> Self {
        self.uri.user = Some(user);
        self
    }

    pub fn host(mut self, host_port: HostPort) -> Self {
        self.uri.host_port = host_port;
        self
    }

    pub fn user_param(mut self, user_param: &str) -> Self {
        self.uri.user_param = Some(user_param.into());
        self
    }

    pub fn method_param(mut self, method_param: &str) -> Self {
        self.uri.method_param = Some(method_param.into());
        self
    }
    pub fn transport_param(
        mut self,
        transport_param: TransportProtocol,
    ) -> Self {
        self.uri.transport_param = Some(transport_param);
        self
    }

    pub fn ttl_param(mut self, ttl_param: &str) -> Self {
        self.uri.ttl_param = Some(ttl_param.into());
        self
    }
    pub fn lr_param(mut self, lr_param: &str) -> Self {
        self.uri.lr_param = Some(lr_param.into());
        self
    }

    pub fn maddr_param(mut self, maddr_param: &str) -> Self {
        self.uri.maddr_param = Some(maddr_param.into());
        self
    }

    pub fn params(mut self, params: Params) -> Self {
        self.uri.params = Some(params);
        self
    }

    pub fn get(self) -> Uri {
        self.uri
    }
}

// SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NameAddr {
    pub display: Option<ArcStr>,
    pub uri: Uri,
}

impl fmt::Display for NameAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(display) = &self.display {
            write!(f, "{} ", display)?;
        }
        write!(f, "<{}>", self.uri)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Params {
    pub(crate) inner: HashMap<ArcStr, ArcStr>,
}

impl fmt::Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formater = self
            .iter()
            .format_with(";", |it, f| f(&format_args!("{}={}", it.0, it.1)));
        write!(f, "{}", formater)
    }
}

impl From<HashMap<ArcStr, ArcStr>> for Params {
    fn from(value: HashMap<ArcStr, ArcStr>) -> Self {
        Self { inner: value }
    }
}

impl Params {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ArcStr, &ArcStr)> {
        self.inner.iter()
    }

    pub fn set(&mut self, k: ArcStr, v: ArcStr) -> Option<ArcStr> {
        self.inner.insert(k, v)
    }
    pub fn get(&self, k: ArcStr) -> Option<&str> {
        self.inner.get(&k).map(|s| s.as_ref())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    Unknow,
}

impl fmt::Display for SipMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SipMethod::Invite => write!(f, "INVITE"),
            SipMethod::Ack => write!(f, "ACK"),
            SipMethod::Bye => write!(f, "BYE"),
            SipMethod::Cancel => write!(f, "CANCEL"),
            SipMethod::Register => write!(f, "REGISTER"),
            SipMethod::Options => write!(f, "OPTIONS"),
            SipMethod::Info => write!(f, "INFO"),
            SipMethod::Notify => write!(f, "NOTIFY"),
            SipMethod::Subscribe => write!(f, "SUBSCRIBE"),
            SipMethod::Update => write!(f, "UPDATE"),
            SipMethod::Refer => write!(f, "REFER"),
            SipMethod::Prack => write!(f, "PRACK"),
            SipMethod::Message => write!(f, "MESSAGE"),
            SipMethod::Publish => write!(f, "PUBLISH"),
            SipMethod::Unknow => write!(f, "UNKNOW-Method"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TransportProtocol {
    #[default]
    UDP,
    TCP,
    TLS,
    SCTP,
    Unknow,
}

impl TransportProtocol {
    pub fn get_port(&self) -> u16 {
        match self {
            TransportProtocol::UDP
            | TransportProtocol::TCP
            | TransportProtocol::SCTP => 5060,
            TransportProtocol::TLS => 5061,
            TransportProtocol::Unknow => 0,
        }
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportProtocol::UDP => write!(f, "{}", TP_UDP),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum StatusCode {
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
impl StatusCode {
    pub fn reason_phrase(&self) -> &'static str {
        match self {
            // 1xx — Provisional Responses
            StatusCode::Trying => "Trying",
            StatusCode::Ringing => "Ringing",
            StatusCode::CallIsBeingForwarded => "Call Is Being Forwarded",
            StatusCode::Queued => "Queued",
            StatusCode::SessionProgress => "Session Progress",

            // 2xx — Successful Responses
            StatusCode::Ok => "OK",
            StatusCode::Accepted => "Accepted",
            StatusCode::NoNotification => "No Notification",

            // 3xx — Redirection Responses
            StatusCode::MultipleChoices => "Multiple Choices",
            StatusCode::MovedPermanently => "Moved Permanently",
            StatusCode::MovedTemporarily => "Moved Temporarily",
            StatusCode::UseProxy => "Use Proxy",
            StatusCode::AlternativeService => "Alternative Service",

            // 4xx — Client Failure Responses
            StatusCode::BadRequest => "Bad Request",
            StatusCode::Unauthorized => "Unauthorized",
            StatusCode::PaymentRequired => "Payment Required",
            StatusCode::Forbidden => "Forbidden",
            StatusCode::NotFound => "Not Found",
            StatusCode::MethodNotAllowed => "Method Not Allowed",
            StatusCode::NotAcceptable => "Not Acceptable",
            StatusCode::ProxyAuthenticationRequired => {
                "Proxy Authentication Required"
            }
            StatusCode::RequestTimeout => "Request Timeout",
            StatusCode::Gone => "Gone",
            StatusCode::RequestEntityTooLarge => "Request Entity Too Large",
            StatusCode::RequestUriTooLong => "Request-URI Too Long",
            StatusCode::UnsupportedMediaType => "Unsupported Media Type",
            StatusCode::UnsupportedUriScheme => "Unsupported URI Scheme",
            StatusCode::BadExtension => "Bad Extension",
            StatusCode::ExtensionRequired => "Extension Required",
            StatusCode::IntervalTooBrief => "Interval Too Brief",
            StatusCode::TemporarilyUnavailable => "Temporarily Unavailable",
            StatusCode::CallOrTransactionDoesNotExist => {
                "Call/Transaction Does Not Exist"
            }
            StatusCode::LoopDetected => "Loop Detected",
            StatusCode::TooManyHops => "Too Many Hops",
            StatusCode::AddressIncomplete => "Address Incomplete",
            StatusCode::Ambiguous => "Ambiguous",
            StatusCode::BusyHere => "Busy Here",
            StatusCode::RequestTerminated => "Request Terminated",
            StatusCode::NotAcceptableHere => "Not Acceptable Here",
            StatusCode::RequestPending => "Request Pending",
            StatusCode::Undecipherable => "Undecipherable",

            // 5xx — Server Failure Responses
            StatusCode::ServerInternalError => "Server Internal Error",
            StatusCode::NotImplemented => "Not Implemented",
            StatusCode::BadGateway => "Bad Gateway",
            StatusCode::ServiceUnavailable => "Service Unavailable",
            StatusCode::ServerTimeout => "Server Time-out",
            StatusCode::VersionNotSupported => "Version Not Supported",
            StatusCode::MessageTooLarge => "Message Too Large",

            // 6xx — Global Failure Responses
            StatusCode::BusyEverywhere => "Busy Everywhere",
            StatusCode::Decline => "Decline",
            StatusCode::DoesNotExistAnywhere => "Does Not Exist Anywhere",
            StatusCode::NotAcceptableAnywhere => "Not Acceptable",

            // Unknown or custom status
            _ => "Unknown",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            StatusCode::Trying => "100",
            StatusCode::Ringing => "180",
            StatusCode::CallIsBeingForwarded => "181",
            StatusCode::Queued => "182",
            StatusCode::SessionProgress => "183",
            StatusCode::EarlyDialogTerminated => "199",
            StatusCode::Ok => "200",
            StatusCode::Accepted => "202",
            StatusCode::NoNotification => "204",
            StatusCode::MultipleChoices => "300",
            StatusCode::MovedPermanently => "301",
            StatusCode::MovedTemporarily => "302",
            StatusCode::UseProxy => "305",
            StatusCode::AlternativeService => "380",
            StatusCode::BadRequest => "400",
            StatusCode::Unauthorized => "401",
            StatusCode::PaymentRequired => "402",
            StatusCode::Forbidden => "403",
            StatusCode::NotFound => "404",
            StatusCode::MethodNotAllowed => "405",
            StatusCode::NotAcceptable => "406",
            StatusCode::ProxyAuthenticationRequired => "407",
            StatusCode::RequestTimeout => "408",
            StatusCode::Conflict => "409",
            StatusCode::Gone => "410",
            StatusCode::LengthRequired => "411",
            StatusCode::ConditionalRequestFailed => "412",
            StatusCode::RequestEntityTooLarge => "413",
            StatusCode::RequestUriTooLong => "414",
            StatusCode::UnsupportedMediaType => "415",
            StatusCode::UnsupportedUriScheme => "416",
            StatusCode::UnknownResourcePriority => "417",
            StatusCode::BadExtension => "420",
            StatusCode::ExtensionRequired => "421",
            StatusCode::SessionTimerTooSmall => "422",
            StatusCode::IntervalTooBrief => "423",
            StatusCode::BadLocationInformation => "424",
            StatusCode::UseIdentityHeader => "428",
            StatusCode::ProvideReferrerHeader => "429",
            StatusCode::FlowFailed => "430",
            StatusCode::AnonimityDisallowed => "433",
            StatusCode::BadIdentityInfo => "436",
            StatusCode::UnsupportedCertificate => "437",
            StatusCode::InvalidIdentityHeader => "438",
            StatusCode::FirstHopLacksOutboundSupport => "439",
            StatusCode::MaxBreadthExceeded => "440",
            StatusCode::BadInfoPackage => "469",
            StatusCode::ConsentNeeded => "470",
            StatusCode::TemporarilyUnavailable => "480",
            StatusCode::CallOrTransactionDoesNotExist => "481",
            StatusCode::LoopDetected => "482",
            StatusCode::TooManyHops => "483",
            StatusCode::AddressIncomplete => "484",
            StatusCode::Ambiguous => "485",
            StatusCode::BusyHere => "486",
            StatusCode::RequestTerminated => "487",
            StatusCode::NotAcceptableHere => "488",
            StatusCode::BadEvent => "489",
            StatusCode::RequestUpdated => "490",
            StatusCode::RequestPending => "491",
            StatusCode::Undecipherable => "493",
            StatusCode::SecurityAgreementNeeded => "494",
            StatusCode::ServerInternalError => "500",
            StatusCode::NotImplemented => "501",
            StatusCode::BadGateway => "502",
            StatusCode::ServiceUnavailable => "503",
            StatusCode::ServerTimeout => "504",
            StatusCode::VersionNotSupported => "505",
            StatusCode::MessageTooLarge => "513",
            StatusCode::PushNotificationServiceNotSupported => "555",
            StatusCode::PreconditionFailure => "580",
            StatusCode::BusyEverywhere => "600",
            StatusCode::Decline => "603",
            StatusCode::DoesNotExistAnywhere => "604",
            StatusCode::NotAcceptableAnywhere => "606",
            StatusCode::Unwanted => "607",
            StatusCode::Rejected => "608",
            StatusCode::Unknow => "Unknow-Code",
        }
    }

    pub fn is_provisional(&self) -> bool {
        match self {
            StatusCode::Trying
            | StatusCode::Ringing
            | StatusCode::CallIsBeingForwarded
            | StatusCode::Queued
            | StatusCode::SessionProgress
            | StatusCode::EarlyDialogTerminated => true,
            _ => false,
        }
    }

    pub fn into_u32(self) -> u32 {
        self as u32
    }
}

impl From<&[u8]> for StatusCode {
    fn from(value: &[u8]) -> Self {
        match value {
            b"100" => StatusCode::Trying,
            b"180" => StatusCode::Ringing,
            b"181" => StatusCode::CallIsBeingForwarded,
            b"182" => StatusCode::Queued,
            b"183" => StatusCode::SessionProgress,
            b"199" => StatusCode::EarlyDialogTerminated,
            b"200" => StatusCode::Ok,
            b"202" => StatusCode::Accepted,
            b"204" => StatusCode::NoNotification,
            b"300" => StatusCode::MultipleChoices,
            b"301" => StatusCode::MovedPermanently,
            b"302" => StatusCode::MovedTemporarily,
            b"305" => StatusCode::UseProxy,
            b"380" => StatusCode::AlternativeService,
            b"400" => StatusCode::BadRequest,
            b"401" => StatusCode::Unauthorized,
            b"402" => StatusCode::PaymentRequired,
            b"403" => StatusCode::Forbidden,
            b"404" => StatusCode::NotFound,
            b"405" => StatusCode::MethodNotAllowed,
            b"406" => StatusCode::NotAcceptable,
            b"407" => StatusCode::ProxyAuthenticationRequired,
            b"408" => StatusCode::RequestTimeout,
            b"409" => StatusCode::Conflict,
            b"410" => StatusCode::Gone,
            b"411" => StatusCode::LengthRequired,
            b"412" => StatusCode::ConditionalRequestFailed,
            b"413" => StatusCode::RequestEntityTooLarge,
            b"414" => StatusCode::RequestUriTooLong,
            b"415" => StatusCode::UnsupportedMediaType,
            b"416" => StatusCode::UnsupportedUriScheme,
            b"417" => StatusCode::UnknownResourcePriority,
            b"420" => StatusCode::BadExtension,
            b"421" => StatusCode::ExtensionRequired,
            b"422" => StatusCode::SessionTimerTooSmall,
            b"423" => StatusCode::IntervalTooBrief,
            b"424" => StatusCode::BadLocationInformation,
            b"428" => StatusCode::UseIdentityHeader,
            b"429" => StatusCode::ProvideReferrerHeader,
            b"430" => StatusCode::FlowFailed,
            b"433" => StatusCode::AnonimityDisallowed,
            b"436" => StatusCode::BadIdentityInfo,
            b"437" => StatusCode::UnsupportedCertificate,
            b"438" => StatusCode::InvalidIdentityHeader,
            b"439" => StatusCode::FirstHopLacksOutboundSupport,
            b"440" => StatusCode::MaxBreadthExceeded,
            b"469" => StatusCode::BadInfoPackage,
            b"470" => StatusCode::ConsentNeeded,
            b"480" => StatusCode::TemporarilyUnavailable,
            b"481" => StatusCode::CallOrTransactionDoesNotExist,
            b"482" => StatusCode::LoopDetected,
            b"483" => StatusCode::TooManyHops,
            b"484" => StatusCode::AddressIncomplete,
            b"485" => StatusCode::Ambiguous,
            b"486" => StatusCode::BusyHere,
            b"487" => StatusCode::RequestTerminated,
            b"488" => StatusCode::NotAcceptableHere,
            b"489" => StatusCode::BadEvent,
            b"490" => StatusCode::RequestUpdated,
            b"491" => StatusCode::RequestPending,
            b"493" => StatusCode::Undecipherable,
            b"494" => StatusCode::SecurityAgreementNeeded,
            b"500" => StatusCode::ServerInternalError,
            b"501" => StatusCode::NotImplemented,
            b"502" => StatusCode::BadGateway,
            b"503" => StatusCode::ServiceUnavailable,
            b"504" => StatusCode::ServerTimeout,
            b"505" => StatusCode::VersionNotSupported,
            b"513" => StatusCode::MessageTooLarge,
            b"555" => StatusCode::PushNotificationServiceNotSupported,
            b"580" => StatusCode::PreconditionFailure,
            b"600" => StatusCode::BusyEverywhere,
            b"603" => StatusCode::Decline,
            b"604" => StatusCode::DoesNotExistAnywhere,
            b"606" => StatusCode::NotAcceptableAnywhere,
            b"607" => StatusCode::Unwanted,
            b"608" => StatusCode::Rejected,
            _ => StatusCode::Unknow,
        }
    }
}

impl From<StatusCode> for StatusLine {
    fn from(value: StatusCode) -> Self {
        StatusLine {
            code: value,
            rphrase: value.reason_phrase().into(),
        }
    }
}
