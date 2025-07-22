/// Reason phrase for the `Trying` status code (100).
pub const REASON_TRYING: &str = "Trying";
/// Reason phrase for the `Ringing` status code (180).
pub const REASON_RINGING: &str = "Ringing";
/// Reason phrase for the `Call Is Being Forwarded` status code (181).
pub const REASON_CALL_IS_BEING_FORWARDED: &str = "Call Is Being Forwarded";
/// Reason phrase for the `Queued` status code (182).
pub const REASON_QUEUED: &str = "Queued";
/// Reason phrase for the `Session Progress` status code (183).
pub const REASON_SESSION_PROGRESS: &str = "Session Progress";
/// Reason phrase for the `OK` status code (200).
pub const REASON_OK: &str = "OK";
/// Reason phrase for the `Accepted` status code (202).
pub const REASON_ACCEPTED: &str = "Accepted";
/// Reason phrase for the `No Notification` status code (204).
pub const REASON_NO_NOTIFICATION: &str = "No Notification";
/// Reason phrase for the `Multiple Choices` status code (300).
pub const REASON_MULTIPLE_CHOICES: &str = "Multiple Choices";
/// Reason phrase for the `Moved Permanently` status code (301).
pub const REASON_MOVED_PERMANENTLY: &str = "Moved Permanently";
/// Reason phrase for the `Moved Temporarily` status code (302).
pub const REASON_MOVED_TEMPORARILY: &str = "Moved Temporarily";
/// Reason phrase for the `Use Proxy` status code (305).
pub const REASON_USE_PROXY: &str = "Use Proxy";
/// Reason phrase for the `Alternative Service` status code (380).
pub const REASON_ALTERNATIVE_SERVICE: &str = "Alternative Service";
/// Reason phrase for the `Bad Request` status code (400).
pub const REASON_BAD_REQUEST: &str = "Bad Request";
/// Reason phrase for the `Unauthorized` status code (401).
pub const REASON_UNAUTHORIZED: &str = "Unauthorized";
/// Reason phrase for the `Payment Required` status code (402).
pub const REASON_PAYMENT_REQUIRED: &str = "Payment Required";
/// Reason phrase for the `Forbidden` status code (403).
pub const REASON_FORBIDDEN: &str = "Forbidden";
/// Reason phrase for the `Not Found` status code (404).
pub const REASON_NOT_FOUND: &str = "Not Found";
/// Reason phrase for the `Method Not Allowed` status code (405).
pub const REASON_METHOD_NOT_ALLOWED: &str = "Method Not Allowed";
/// Reason phrase for the `Not Acceptable` status code (406).
pub const REASON_NOT_ACCEPTABLE: &str = "Not Acceptable";
/// Reason phrase for the `Proxy Authentication Required` status code (407).
pub const REASON_PROXY_AUTHENTICATION_REQUIRED: &str = "Proxy Authentication Required";
/// Reason phrase for the `Request Timeout` status code (408).
pub const REASON_REQUEST_TIMEOUT: &str = "Request Timeout";
/// Reason phrase for the `Gone` status code (410).
pub const REASON_GONE: &str = "Gone";
/// Reason phrase for the `Request Entity Too Large` status code (413).
pub const REASON_REQUEST_ENTITY_TOO_LARGE: &str = "Request Entity Too Large";
/// Reason phrase for the `Request-URI Too Long` status code (414).
pub const REASON_REQUEST_URI_TOO_LONG: &str = "Request-URI Too Long";
/// Reason phrase for the `Unsupported Media Type` status code (415).
pub const REASON_UNSUPPORTED_MEDIA_TYPE: &str = "Unsupported Media Type";
/// Reason phrase for the `Unsupported URI Scheme` status code (416).
pub const REASON_UNSUPPORTED_URI_SCHEME: &str = "Unsupported URI Scheme";
/// Reason phrase for the `Bad Extension` status code (420).
pub const REASON_BAD_EXTENSION: &str = "Bad Extension";
/// Reason phrase for the `Extension Required` status code (421).
pub const REASON_EXTENSION_REQUIRED: &str = "Extension Required";
/// Reason phrase for the `Interval Too Brief` status code (423).
pub const REASON_INTERVAL_TOO_BRIEF: &str = "Interval Too Brief";
/// Reason phrase for the `Temporarily Unavailable` status code (480).
pub const REASON_TEMPORARILY_UNAVAILABLE: &str = "Temporarily Unavailable";
/// Reason phrase for the `Call/Transaction Does Not Exist` status code (481).
pub const REASON_CALL_OR_TRANSACTION_DOES_NOT_EXIST: &str = "Call/Transaction Does Not Exist";
/// Reason phrase for the `Loop Detected` status code (482).
pub const REASON_LOOP_DETECTED: &str = "Loop Detected";
/// Reason phrase for the `Too Many Hops` status code (483).
pub const REASON_TOO_MANY_HOPS: &str = "Too Many Hops";
/// Reason phrase for the `Address Incomplete` status code (484).
pub const REASON_ADDRESS_INCOMPLETE: &str = "Address Incomplete";
/// Reason phrase for the `Ambiguous` status code (485).
pub const REASON_AMBIGUOUS: &str = "Ambiguous";
/// Reason phrase for the `Busy Here` status code (486).
pub const REASON_BUSY_HERE: &str = "Busy Here";
/// Reason phrase for the `Request Terminated` status code (487).
pub const REASON_REQUEST_TERMINATED: &str = "Request Terminated";
/// Reason phrase for the `Not Acceptable Here` status code (488).
pub const REASON_NOT_ACCEPTABLE_HERE: &str = "Not Acceptable Here";
/// Reason phrase for the `Request Pending` status code (491).
pub const REASON_REQUEST_PENDING: &str = "Request Pending";
/// Reason phrase for the `Undecipherable` status code (493).
pub const REASON_UNDECIPHERABLE: &str = "Undecipherable";
/// Reason phrase for the `Server Internal Error` status code (500).
pub const REASON_SERVER_INTERNAL_ERROR: &str = "Server Internal Error";
/// Reason phrase for the `Not Implemented` status code (501).
pub const REASON_NOT_IMPLEMENTED: &str = "Not Implemented";
/// Reason phrase for the `Bad Gateway` status code (502).
pub const REASON_BAD_GATEWAY: &str = "Bad Gateway";
/// Reason phrase for the `Service Unavailable` status code (503).
pub const REASON_SERVICE_UNAVAILABLE: &str = "Service Unavailable";
/// Reason phrase for the `Server Time-out` status code (504).
pub const REASON_SERVER_TIMEOUT: &str = "Server Time-out";
/// Reason phrase for the `Version Not Supported` status code (505).
pub const REASON_VERSION_NOT_SUPPORTED: &str = "Version Not Supported";
/// Reason phrase for the `Message Too Large` status code (513).
pub const REASON_MESSAGE_TOO_LARGE: &str = "Message Too Large";
/// Reason phrase for the `Busy Everywhere` status code (600).
pub const REASON_BUSY_EVERYWHERE: &str = "Busy Everywhere";
/// Reason phrase for the `Decline` status code (603).
pub const REASON_DECLINE: &str = "Decline";
/// Reason phrase for the `Does Not Exist Anywhere` status code (604).
pub const REASON_DOES_NOT_EXIST_ANYWHERE: &str = "Does Not Exist Anywhere";
/// Reason phrase for the `Not Acceptable` status code (606).
pub const REASON_NOT_ACCEPTABLE_ANYWHERE: &str = REASON_NOT_ACCEPTABLE;
/// Reason phrase for the `Rejected` status code (608).
pub const REASON_REJECTED: &str = "Rejected";

/// An SIP status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[repr(i32)]
pub enum StatusCode {
    /// `Trying` status code.
    Trying = 100,
    /// `Ringing` status code.
    Ringing = 180,
    /// `Call Is Being Forwarded` status code.
    CallIsBeingForwarded = 181,
    /// `Queued` status code.
    Queued = 182,
    /// `Session Progress` status code.
    SessionProgress = 183,
    /// `Early Dialog Terminated` status code.
    EarlyDialogTerminated = 199,

    /// `OK` status code.
    Ok = 200,
    /// `Accepted` status code.
    Accepted = 202,
    /// `No Notification` status code.
    NoNotification = 204,

    /// `Multiple Choices` status code.
    MultipleChoices = 300,
    /// `Moved Permanently` status code.
    MovedPermanently = 301,
    /// `Moved Temporarily` status code.
    MovedTemporarily = 302,
    /// `Use Proxy` status code.
    UseProxy = 305,
    /// `Alternative Service` status code.
    AlternativeService = 380,

    /// `Bad Request` status code.
    BadRequest = 400,
    /// `Unauthorized` status code.
    Unauthorized = 401,
    /// `Payment Required` status code.
    PaymentRequired = 402,
    /// `Forbidden` status code.
    Forbidden = 403,
    /// `Not Found` status code.
    NotFound = 404,
    /// `SipMethod Not Allowed` status code.
    MethodNotAllowed = 405,
    /// `Not Acceptable` status code.
    NotAcceptable = 406,
    /// `Proxy Authentication Required` status code.
    ProxyAuthenticationRequired = 407,
    /// `Request Timeout` status code.
    RequestTimeout = 408,
    /// `Conflict` status code.
    Conflict = 409,
    /// `Gone` status code.
    Gone = 410,
    /// `Length Required` status code.
    LengthRequired = 411,
    /// `Conditional Request Failed` status code.
    ConditionalRequestFailed = 412,
    /// `Request Entity Too Large` status code.
    RequestEntityTooLarge = 413,
    /// `Request URI Too Long` status code.
    RequestUriTooLong = 414,
    /// `Unsupported Media Type` status code.
    UnsupportedMediaType = 415,
    /// `Unsupported URI Scheme` status code.
    UnsupportedUriScheme = 416,
    /// `Unknown Resource Priority` status code.
    UnknownResourcePriority = 417,
    /// `Bad Extension` status code.
    BadExtension = 420,
    /// `Extension Required` status code.
    ExtensionRequired = 421,
    /// `Session Timer Too Small` status code.
    SessionTimerTooSmall = 422,
    /// `Interval Too Brief` status code.
    IntervalTooBrief = 423,
    /// `Bad Location Information` status code.
    BadLocationInformation = 424,
    /// `Use Identity Header` status code.
    UseIdentityHeader = 428,
    /// `Provide Referrer Header` status code.
    ProvideReferrerHeader = 429,
    /// `Flow Failed` status code.
    FlowFailed = 430,
    /// `Anonymity Disallowed` status code.
    AnonimityDisallowed = 433,
    /// `Bad Identity Info` status code.
    BadIdentityInfo = 436,
    /// `Unsupported Certificate` status code.
    UnsupportedCertificate = 437,
    /// `Invalid Identity Header` status code.
    InvalidIdentityHeader = 438,
    /// `First Hop Lacks Outbound Support` status code.
    FirstHopLacksOutboundSupport = 439,
    /// `Max Breadth Exceeded` status code.
    MaxBreadthExceeded = 440,
    /// `Bad Info Package` status code.
    BadInfoPackage = 469,
    /// `Consent Needed` status code.
    ConsentNeeded = 470,
    /// `Temporarily Unavailable` status code.
    TemporarilyUnavailable = 480,
    /// `Call or Transaction Does Not Exist` status code.
    CallOrTransactionDoesNotExist = 481,
    /// `Loop Detected` status code.
    LoopDetected = 482,
    /// `Too Many Hops` status code.
    TooManyHops = 483,
    /// `Address Incomplete` status code.
    AddressIncomplete = 484,
    /// `Ambiguous` status code.
    Ambiguous = 485,
    /// `Busy Here` status code.
    BusyHere = 486,
    /// `Request Terminated` status code.
    RequestTerminated = 487,
    /// `Not Acceptable Here` status code.
    NotAcceptableHere = 488,
    /// `Bad Event` status code.
    BadEvent = 489,
    /// `Request Updated` status code.
    RequestUpdated = 490,
    /// `Request Pending` status code.
    RequestPending = 491,
    /// `Undecipherable` status code.
    Undecipherable = 493,
    /// `Security Agreement Needed` status code.
    SecurityAgreementNeeded = 494,

    /// `Server Internal Error` status code.
    ServerInternalError = 500,
    /// `Not Implemented` status code.
    NotImplemented = 501,
    /// `Bad Gateway` status code.
    BadGateway = 502,
    /// `Service Unavailable` status code.
    ServiceUnavailable = 503,
    /// `Server Timeout` status code.
    ServerTimeout = 504,
    /// `Version Not Supported` status code.
    VersionNotSupported = 505,
    /// `Message Too Large` status code.
    MessageTooLarge = 513,
    /// `Push Notification Service Not Supported` status code.
    PushNotificationServiceNotSupported = 555,
    /// `Precondition Failure` status code.
    PreconditionFailure = 580,

    /// `Busy Everywhere` status code.
    BusyEverywhere = 600,
    /// `Decline` status code.
    Decline = 603,
    /// `Does Not Exist Anywhere` status code.
    DoesNotExistAnywhere = 604,
    /// `Not Acceptable Anywhere` status code.
    NotAcceptableAnywhere = 606,
    /// `Unwanted` status code.
    Unwanted = 607,
    /// `Rejected` status code.
    Rejected = 608,

    /// A non-standard or unknown status code.
    Custom(i32),
}

// https://en.wikipedia.org/wiki/List_of_SIP_response_codes
impl StatusCode {
    /// Returns the reason text related to the status code.
    pub const fn reason(&self) -> &'static str {
        match self {
            // 1xx — Provisional Responses
            StatusCode::Trying => REASON_TRYING,
            StatusCode::Ringing => REASON_RINGING,
            StatusCode::CallIsBeingForwarded => REASON_CALL_IS_BEING_FORWARDED,
            StatusCode::Queued => REASON_QUEUED,
            StatusCode::SessionProgress => REASON_SESSION_PROGRESS,

            // 2xx — Successful Responses
            StatusCode::Ok => REASON_OK,
            StatusCode::Accepted => REASON_ACCEPTED,
            StatusCode::NoNotification => REASON_NO_NOTIFICATION,

            // 3xx — Redirection Responses
            StatusCode::MultipleChoices => REASON_MULTIPLE_CHOICES,
            StatusCode::MovedPermanently => REASON_MOVED_PERMANENTLY,
            StatusCode::MovedTemporarily => REASON_MOVED_TEMPORARILY,
            StatusCode::UseProxy => REASON_USE_PROXY,
            StatusCode::AlternativeService => REASON_ALTERNATIVE_SERVICE,

            // 4xx — Client Failure Responses
            StatusCode::BadRequest => REASON_BAD_REQUEST,
            StatusCode::Unauthorized => REASON_UNAUTHORIZED,
            StatusCode::PaymentRequired => REASON_PAYMENT_REQUIRED,
            StatusCode::Forbidden => REASON_FORBIDDEN,
            StatusCode::NotFound => REASON_NOT_FOUND,
            StatusCode::MethodNotAllowed => REASON_METHOD_NOT_ALLOWED,
            StatusCode::NotAcceptable => REASON_NOT_ACCEPTABLE,
            StatusCode::ProxyAuthenticationRequired => REASON_PROXY_AUTHENTICATION_REQUIRED,
            StatusCode::RequestTimeout => REASON_REQUEST_TIMEOUT,
            StatusCode::Gone => REASON_GONE,
            StatusCode::RequestEntityTooLarge => REASON_REQUEST_ENTITY_TOO_LARGE,
            StatusCode::RequestUriTooLong => REASON_REQUEST_URI_TOO_LONG,
            StatusCode::UnsupportedMediaType => REASON_UNSUPPORTED_MEDIA_TYPE,
            StatusCode::UnsupportedUriScheme => REASON_UNSUPPORTED_URI_SCHEME,
            StatusCode::BadExtension => REASON_BAD_EXTENSION,
            StatusCode::ExtensionRequired => REASON_EXTENSION_REQUIRED,
            StatusCode::IntervalTooBrief => REASON_INTERVAL_TOO_BRIEF,
            StatusCode::TemporarilyUnavailable => REASON_TEMPORARILY_UNAVAILABLE,
            StatusCode::CallOrTransactionDoesNotExist => REASON_CALL_OR_TRANSACTION_DOES_NOT_EXIST,
            StatusCode::LoopDetected => REASON_LOOP_DETECTED,
            StatusCode::TooManyHops => REASON_TOO_MANY_HOPS,
            StatusCode::AddressIncomplete => REASON_ADDRESS_INCOMPLETE,
            StatusCode::Ambiguous => REASON_AMBIGUOUS,
            StatusCode::BusyHere => REASON_BUSY_HERE,
            StatusCode::RequestTerminated => REASON_REQUEST_TERMINATED,
            StatusCode::NotAcceptableHere => REASON_NOT_ACCEPTABLE_HERE,
            StatusCode::RequestPending => REASON_REQUEST_PENDING,
            StatusCode::Undecipherable => REASON_UNDECIPHERABLE,

            // 5xx — Server Failure Responses
            StatusCode::ServerInternalError => REASON_SERVER_INTERNAL_ERROR,
            StatusCode::NotImplemented => REASON_NOT_IMPLEMENTED,
            StatusCode::BadGateway => REASON_BAD_GATEWAY,
            StatusCode::ServiceUnavailable => REASON_SERVICE_UNAVAILABLE,
            StatusCode::ServerTimeout => REASON_SERVER_TIMEOUT,
            StatusCode::VersionNotSupported => REASON_VERSION_NOT_SUPPORTED,
            StatusCode::MessageTooLarge => REASON_MESSAGE_TOO_LARGE,

            // 6xx — Global Failure Responses
            StatusCode::BusyEverywhere => REASON_BUSY_EVERYWHERE,
            StatusCode::Decline => REASON_DECLINE,
            StatusCode::DoesNotExistAnywhere => REASON_DOES_NOT_EXIST_ANYWHERE,
            StatusCode::NotAcceptableAnywhere => REASON_NOT_ACCEPTABLE_ANYWHERE,
            StatusCode::Rejected => REASON_REJECTED,

            // Unknown or custom status
            _ => "Unknown",
        }
    }

    /// Returns [`true`] if its status code is provisional (from `100` to `199`), and [`false`]
    /// otherwise.
    #[inline]
    pub const fn is_provisional(&self) -> bool {
        matches!(
            self,
            StatusCode::Trying
                | StatusCode::Ringing
                | StatusCode::CallIsBeingForwarded
                | StatusCode::Queued
                | StatusCode::SessionProgress
                | StatusCode::EarlyDialogTerminated
        )
    }

    #[inline]
    /// Returns [`true`] if its status code is final (from `200` to `699`), and [`false`]
    /// otherwise.
    pub const fn is_final(&self) -> bool {
        !self.is_provisional()
    }

    /// Converts a `StatusCode` into its numeric code.
    pub const fn into_i32(self) -> i32 {
        match self {
            StatusCode::Trying => 100,
            StatusCode::Ringing => 180,
            StatusCode::CallIsBeingForwarded => 181,
            StatusCode::Queued => 182,
            StatusCode::SessionProgress => 183,
            StatusCode::EarlyDialogTerminated => 199,

            StatusCode::Ok => 200,
            StatusCode::Accepted => 202,
            StatusCode::NoNotification => 204,

            StatusCode::MultipleChoices => 300,
            StatusCode::MovedPermanently => 301,
            StatusCode::MovedTemporarily => 302,
            StatusCode::UseProxy => 305,
            StatusCode::AlternativeService => 380,

            StatusCode::BadRequest => 400,
            StatusCode::Unauthorized => 401,
            StatusCode::PaymentRequired => 402,
            StatusCode::Forbidden => 403,
            StatusCode::NotFound => 404,
            StatusCode::MethodNotAllowed => 405,
            StatusCode::NotAcceptable => 406,
            StatusCode::ProxyAuthenticationRequired => 407,
            StatusCode::RequestTimeout => 408,
            StatusCode::Conflict => 409,
            StatusCode::Gone => 410,
            StatusCode::LengthRequired => 411,
            StatusCode::ConditionalRequestFailed => 412,
            StatusCode::RequestEntityTooLarge => 413,
            StatusCode::RequestUriTooLong => 414,
            StatusCode::UnsupportedMediaType => 415,
            StatusCode::UnsupportedUriScheme => 416,
            StatusCode::UnknownResourcePriority => 417,
            StatusCode::BadExtension => 420,
            StatusCode::ExtensionRequired => 421,
            StatusCode::SessionTimerTooSmall => 422,
            StatusCode::IntervalTooBrief => 423,
            StatusCode::BadLocationInformation => 424,
            StatusCode::UseIdentityHeader => 428,
            StatusCode::ProvideReferrerHeader => 429,
            StatusCode::FlowFailed => 430,
            StatusCode::AnonimityDisallowed => 433,
            StatusCode::BadIdentityInfo => 436,
            StatusCode::UnsupportedCertificate => 437,
            StatusCode::InvalidIdentityHeader => 438,
            StatusCode::FirstHopLacksOutboundSupport => 439,
            StatusCode::MaxBreadthExceeded => 440,
            StatusCode::BadInfoPackage => 469,
            StatusCode::ConsentNeeded => 470,
            StatusCode::TemporarilyUnavailable => 480,
            StatusCode::CallOrTransactionDoesNotExist => 481,
            StatusCode::LoopDetected => 482,
            StatusCode::TooManyHops => 483,
            StatusCode::AddressIncomplete => 484,
            StatusCode::Ambiguous => 485,
            StatusCode::BusyHere => 486,
            StatusCode::RequestTerminated => 487,
            StatusCode::NotAcceptableHere => 488,
            StatusCode::BadEvent => 489,
            StatusCode::RequestUpdated => 490,
            StatusCode::RequestPending => 491,
            StatusCode::Undecipherable => 493,
            StatusCode::SecurityAgreementNeeded => 494,

            StatusCode::ServerInternalError => 500,
            StatusCode::NotImplemented => 501,
            StatusCode::BadGateway => 502,
            StatusCode::ServiceUnavailable => 503,
            StatusCode::ServerTimeout => 504,
            StatusCode::VersionNotSupported => 505,
            StatusCode::MessageTooLarge => 513,
            StatusCode::PushNotificationServiceNotSupported => 555,
            StatusCode::PreconditionFailure => 580,

            StatusCode::BusyEverywhere => 600,
            StatusCode::Decline => 603,
            StatusCode::DoesNotExistAnywhere => 604,
            StatusCode::NotAcceptableAnywhere => 606,
            StatusCode::Unwanted => 607,
            StatusCode::Rejected => 608,
            StatusCode::Custom(n) => n,
        }
    }
}

impl From<i32> for StatusCode {
    fn from(value: i32) -> Self {
        match value {
            100 => StatusCode::Trying,
            180 => StatusCode::Ringing,
            181 => StatusCode::CallIsBeingForwarded,
            182 => StatusCode::Queued,
            183 => StatusCode::SessionProgress,
            199 => StatusCode::EarlyDialogTerminated,
            200 => StatusCode::Ok,
            202 => StatusCode::Accepted,
            204 => StatusCode::NoNotification,
            300 => StatusCode::MultipleChoices,
            301 => StatusCode::MovedPermanently,
            302 => StatusCode::MovedTemporarily,
            305 => StatusCode::UseProxy,
            380 => StatusCode::AlternativeService,
            400 => StatusCode::BadRequest,
            401 => StatusCode::Unauthorized,
            402 => StatusCode::PaymentRequired,
            403 => StatusCode::Forbidden,
            404 => StatusCode::NotFound,
            405 => StatusCode::MethodNotAllowed,
            406 => StatusCode::NotAcceptable,
            407 => StatusCode::ProxyAuthenticationRequired,
            408 => StatusCode::RequestTimeout,
            409 => StatusCode::Conflict,
            410 => StatusCode::Gone,
            411 => StatusCode::LengthRequired,
            412 => StatusCode::ConditionalRequestFailed,
            413 => StatusCode::RequestEntityTooLarge,
            414 => StatusCode::RequestUriTooLong,
            415 => StatusCode::UnsupportedMediaType,
            416 => StatusCode::UnsupportedUriScheme,
            417 => StatusCode::UnknownResourcePriority,
            420 => StatusCode::BadExtension,
            421 => StatusCode::ExtensionRequired,
            422 => StatusCode::SessionTimerTooSmall,
            423 => StatusCode::IntervalTooBrief,
            424 => StatusCode::BadLocationInformation,
            428 => StatusCode::UseIdentityHeader,
            429 => StatusCode::ProvideReferrerHeader,
            430 => StatusCode::FlowFailed,
            433 => StatusCode::AnonimityDisallowed,
            436 => StatusCode::BadIdentityInfo,
            437 => StatusCode::UnsupportedCertificate,
            438 => StatusCode::InvalidIdentityHeader,
            439 => StatusCode::FirstHopLacksOutboundSupport,
            440 => StatusCode::MaxBreadthExceeded,
            469 => StatusCode::BadInfoPackage,
            470 => StatusCode::ConsentNeeded,
            480 => StatusCode::TemporarilyUnavailable,
            481 => StatusCode::CallOrTransactionDoesNotExist,
            482 => StatusCode::LoopDetected,
            483 => StatusCode::TooManyHops,
            484 => StatusCode::AddressIncomplete,
            485 => StatusCode::Ambiguous,
            486 => StatusCode::BusyHere,
            487 => StatusCode::RequestTerminated,
            488 => StatusCode::NotAcceptableHere,
            489 => StatusCode::BadEvent,
            490 => StatusCode::RequestUpdated,
            491 => StatusCode::RequestPending,
            493 => StatusCode::Undecipherable,
            494 => StatusCode::SecurityAgreementNeeded,
            500 => StatusCode::ServerInternalError,
            501 => StatusCode::NotImplemented,
            502 => StatusCode::BadGateway,
            503 => StatusCode::ServiceUnavailable,
            504 => StatusCode::ServerTimeout,
            505 => StatusCode::VersionNotSupported,
            513 => StatusCode::MessageTooLarge,
            555 => StatusCode::PushNotificationServiceNotSupported,
            580 => StatusCode::PreconditionFailure,
            600 => StatusCode::BusyEverywhere,
            603 => StatusCode::Decline,
            604 => StatusCode::DoesNotExistAnywhere,
            606 => StatusCode::NotAcceptableAnywhere,
            607 => StatusCode::Unwanted,
            608 => StatusCode::Rejected,
            other => StatusCode::Custom(other),
        }
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
            other => {
                let code = std::str::from_utf8(other).unwrap_or("0").parse::<i32>().unwrap();
                StatusCode::Custom(code)
            }
        }
    }
}
