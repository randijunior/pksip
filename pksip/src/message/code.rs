macro_rules! reason_phrases {
    ($($reason_const:ident, $name:literal),*$(,)?) => (
        $(
            const $reason_const: std::sync::LazyLock<crate::ArcStr> =
                std::sync::LazyLock::new(|| $name.into());
        )*
    );
}

// -----------------------------------------------------------------------------
// 1xx – Provisional Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    TRYING, "Trying",
    RINGING, "Ringing",
    CALL_IS_BEING_FORWARDED, "Call Is Being Forwarded",
    QUEUED, "Queued",
    SESSION_PROGRESS, "Session Progress",
    EARLY_DIALOG_TERMINATED, "Early Dialog Terminated",
}

// -----------------------------------------------------------------------------
// 2xx – Successful Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    OK, "OK",
    ACCEPTED, "Accepted",
    NO_NOTIFICATION, "No Notification",
    MULTIPLE_CHOICES, "Multiple Choices",
}

// -----------------------------------------------------------------------------
// 3xx – Redirection Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    MOVED_PERMANENTLY, "Moved Permanently",
    MOVED_TEMPORARILY, "Moved Temporarily",
    USE_PROXY, "Use Proxy",
    ALTERNATIVE_SERVICE, "Alternative Service",
}

// -----------------------------------------------------------------------------
// 4xx – Client Failure Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    BAD_REQUEST, "Bad Request",
    UNAUTHORIZED, "Unauthorized",
    PAYMENT_REQUIRED, "Payment Required",
    FORBIDDEN, "Forbidden",
    NOT_FOUND, "Not Found",
    METHOD_NOT_ALLOWED, "SipMethod Not Allowed",
    NOT_ACCEPTABLE, "Not Acceptable",
    PROXY_AUTHENTICATION_REQUIRED, "Proxy Authentication Required",
    REQUEST_TIMEOUT, "Request Timeout",
    CONFLICT, "Conflict",
    GONE, "Gone",
    LENGTH_REQUIRED, "Length Required",
    CONDITIONAL_REQUEST_FAILED, "Conditional Request Failed",
    REQUEST_ENTITY_TOO_LARGE, "Request Entity Too Large",
    REQUEST_URI_TOO_LONG, "Request URI Too Long",
    UNSUPPORTED_MEDIA_TYPE, "Unsupported Media Type",
    UNSUPPORTED_URI_SCHEME, "Unsupported URI Scheme",
    UNKNOWN_RESOURCE_PRIORITY, "Unknown Resource Priority",
    BAD_EXTENSION, "Bad Extension",
    EXTENSION_REQUIRED, "Extension Required",
    SESSION_INTERVAL_TOO_SMALL, "Session Interval Too Small",
    INTERVAL_TOO_BRIEF, "Interval Too Brief",
    BAD_LOCATION_INFORMATION, "Bad Location Information",
    USE_IDENTITY_HEADER, "Use Identity Header",
    PROVIDE_REFERRER_IDENTITY, "Provide Referrer Identity",
    FLOW_FAILED, "Flow Failed",
    ANONYMITY_DISALLOWED, "Anonymity Disallowed",
    BAD_IDENTITY_INFO, "Bad Identity Info",
    UNSUPPORTED_CERTIFICATE, "Unsupported Certificate",
    INVALID_IDENTITY_HEADER, "Invalid Identity Header",
    FIRST_HOP_LACKS_OUTBOUND_SUPPORT, "First Hop Lacks Outbound Support",
    MAX_BREADTH_EXCEEDED, "Max Breadth Exceeded",
    BAD_INFO_PACKAGE, "Bad Info Package",
    CONSENT_NEEDED, "Consent Needed",
    TEMPORARILY_UNAVAILABLE, "Temporarily Unavailable",
    CALL_OR_TRANSACTION_DOES_NOT_EXIST, "Call or Transaction Does Not Exist",
    LOOP_DETECTED, "Loop Detected",
    TOO_MANY_HOPS, "Too Many Hops",
    ADDRESS_INCOMPLETE, "Address Incomplete",
    AMBIGUOUS, "Ambiguous",
    BUSY_HERE, "Busy Here",
    REQUEST_TERMINATED, "Request Terminated",
    NOT_ACCEPTABLE_HERE, "Not Acceptable Here",
    BAD_EVENT, "Bad Event",
    REQUEST_UPDATED, "Request Updated",
    REQUEST_PENDING, "Request Pending",
    UNDECIPHERABLE, "Undecipherable",
    SECURITY_AGREEMENT_REQUIRED, "Security Agreement Required",
}

// -----------------------------------------------------------------------------
// 5xx – Server Failure Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    SERVER_INTERNAL_ERROR, "Server Internal Error",
    NOT_IMPLEMENTED, "Not Implemented",
    BAD_GATEWAY, "Bad Gateway",
    SERVICE_UNAVAILABLE, "Service Unavailable",
    SERVER_TIMEOUT, "Server Timeout",
    VERSION_NOT_SUPPORTED, "Version Not Supported",
    MESSAGE_TOO_LARGE, "Message Too Large",
    PUSH_NOTIFICATION_SERVICE_NOT_SUPPORTED, "Push Notification Service Not Supported",
    PRECONDITION_FAILURE, "Precondition Failure",
}

// -----------------------------------------------------------------------------
// 6xx – Global Failure Responses
// -----------------------------------------------------------------------------
reason_phrases! {
    BUSY_EVERYWHERE, "Busy Everywhere",
    DECLINE, "Decline",
    DOES_NOT_EXIST_ANYWHERE, "Does Not Exist Anywhere",
    NOT_ACCEPTABLE_ANYWHERE, "Not Acceptable Anywhere",
    UNWANTED, "Unwanted",
    REJECTED, "Rejected",
}

/// Classifies SIP status codes into categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeClass {
    /// Provisional responses (1xx)
    Provisional,
    /// Successful responses (2xx)
    Success,
    /// Redirection responses (3xx)
    Redirection,
    /// Client failure responses (4xx)
    ClientError,
    /// Server failure responses (5xx)
    ServerError,
    /// Global failure responses (6xx)
    GlobalFailure,
}

/// Status Code enum for SIP messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
#[repr(u16)]
pub enum StatusCode {
    ///`Trying` status code.
    Trying = 100,
    ///`Ringing` status code.
    Ringing = 180,
    ///`Call Is Being Forwarded` status code.
    CallIsBeingForwarded = 181,
    ///`Queued` status code.
    Queued = 182,
    ///`InvSession Progress` status code.
    SessionProgress = 183,
    ///`Early Dialog Terminated` status code.
    EarlyDialogTerminated = 199,
    ///`OK` status code.
    Ok = 200,
    ///`Accepted` status code.
    Accepted = 202,
    ///`No Notification` status code.
    NoNotification = 204,
    ///`Multiple Choices` status code.
    MultipleChoices = 300,
    ///`Moved Permanently` status code.
    MovedPermanently = 301,
    ///`Moved Temporarily` status code.
    MovedTemporarily = 302,
    ///`Use Proxy` status code.
    UseProxy = 305,
    ///`Alternative Service` status code.
    AlternativeService = 380,
    ///`Bad Request` status code.
    BadRequest = 400,
    ///`Unauthorized` status code.
    Unauthorized = 401,
    ///`Payment Required` status code.
    PaymentRequired = 402,
    ///`Forbidden` status code.
    Forbidden = 403,
    ///`Not Found` status code.
    NotFound = 404,
    ///`SipMethod Not Allowed` status code.
    MethodNotAllowed = 405,
    ///`Not Acceptable` status code.
    NotAcceptable = 406,
    ///`Proxy Authentication Required` status code.
    ProxyAuthenticationRequired = 407,
    ///`Request Timeout` status code.
    RequestTimeout = 408,
    ///`Conflict` status code.
    Conflict = 409,
    ///`Gone` status code.
    Gone = 410,
    ///`Length Required` status code.
    LengthRequired = 411,
    ///`Conditional Request Failed` status code.
    ConditionalRequestFailed = 412,
    ///`Request Entity Too Large` status code.
    RequestEntityTooLarge = 413,
    ///`Request URI Too Long` status code.
    RequestUriTooLong = 414,
    ///`Unsupported Media Type` status code.
    UnsupportedMediaType = 415,
    ///`Unsupported URI Scheme` status code.
    UnsupportedUriScheme = 416,
    ///`Unknown Resource Priority` status code.
    UnknownResourcePriority = 417,
    ///`Bad Extension` status code.
    BadExtension = 420,
    ///`Extension Required` status code.
    ExtensionRequired = 421,
    ///`InvSession Timer Too Small` status code.
    SessionIntervalTooSmall = 422,
    ///`Interval Too Brief` status code.
    IntervalTooBrief = 423,
    ///`Bad Location Information` status code.
    BadLocationInformation = 424,
    ///`Use Identity Header` status code.
    UseIdentityHeader = 428,
    ///`Provide Referrer Header` status code.
    ProvideReferrerIdentity = 429,
    ///`Flow Failed` status code.
    FlowFailed = 430,
    ///`Anonymity Disallowed` status code.
    AnonymityDisallowed = 433,
    ///`Bad Identity Info` status code.
    BadIdentityInfo = 436,
    ///`Unsupported Certificate` status code.
    UnsupportedCertificate = 437,
    ///`Invalid Identity Header` status code.
    InvalidIdentityHeader = 438,
    ///`First Hop Lacks Outbound Support` status code.
    FirstHopLacksOutboundSupport = 439,
    ///`Max Breadth Exceeded` status code.
    MaxBreadthExceeded = 440,
    ///`Bad Info Package` status code.
    BadInfoPackage = 469,
    ///`Consent Needed` status code.
    ConsentNeeded = 470,
    ///`Temporarily Unavailable` status code.
    TemporarilyUnavailable = 480,
    ///`Call or Transaction Does Not Exist` status code.
    CallOrTransactionDoesNotExist = 481,
    ///`Loop Detected` status code.
    LoopDetected = 482,
    ///`Too Many Hops` status code.
    TooManyHops = 483,
    ///`Address Incomplete` status code.
    AddressIncomplete = 484,
    ///`Ambiguous` status code.
    Ambiguous = 485,
    ///`Busy Here` status code.
    BusyHere = 486,
    ///`Request Terminated` status code.
    RequestTerminated = 487,
    ///`Not Acceptable Here` status code.
    NotAcceptableHere = 488,
    ///`Bad Event` status code.
    BadEvent = 489,
    ///`Request Updated` status code.
    RequestUpdated = 490,
    ///`Request Pending` status code.
    RequestPending = 491,
    ///`Undecipherable` status code.
    Undecipherable = 493,
    ///`Security Agreement Needed` status code.
    SecurityAgreementRequired = 494,
    ///`Server Internal Error` status code.
    ServerInternalError = 500,
    ///`Not Implemented` status code.
    NotImplemented = 501,
    ///`Bad Gateway` status code.
    BadGateway = 502,
    ///`Service Unavailable` status code.
    ServiceUnavailable = 503,
    ///`Server Timeout` status code.
    ServerTimeout = 504,
    ///`Version Not Supported` status code.
    VersionNotSupported = 505,
    ///`SipMessage Too Large` status code.
    MessageTooLarge = 513,
    ///`Push Notification Service Not Supported` status code.
    PushNotificationServiceNotSupported = 555,
    ///`Precondition Failure` status code.
    PreconditionFailure = 580,
    ///`Busy Everywhere` status code.
    BusyEverywhere = 600,
    ///`Decline` status code.
    Decline = 603,
    ///`Does Not Exist Anywhere` status code.
    DoesNotExistAnywhere = 604,
    ///`Not Acceptable Anywhere` status code.
    NotAcceptableAnywhere = 606,
    ///`Unwanted` status code.
    Unwanted = 607,
    ///`Rejected` status code.
    Rejected = 608,
}

impl StatusCode {
    /// Returns the reason text related to the status code.
    pub fn reason(&self) -> crate::ArcStr {
        match self {
            Self::Trying => TRYING.clone(),
            Self::Ringing => RINGING.clone(),
            Self::CallIsBeingForwarded => CALL_IS_BEING_FORWARDED.clone(),
            Self::Queued => QUEUED.clone(),
            Self::SessionProgress => SESSION_PROGRESS.clone(),
            Self::EarlyDialogTerminated => EARLY_DIALOG_TERMINATED.clone(),
            Self::Ok => OK.clone(),
            Self::Accepted => ACCEPTED.clone(),
            Self::NoNotification => NO_NOTIFICATION.clone(),
            Self::MultipleChoices => MULTIPLE_CHOICES.clone(),
            Self::MovedPermanently => MOVED_PERMANENTLY.clone(),
            Self::MovedTemporarily => MOVED_TEMPORARILY.clone(),
            Self::UseProxy => USE_PROXY.clone(),
            Self::AlternativeService => ALTERNATIVE_SERVICE.clone(),
            Self::BadRequest => BAD_REQUEST.clone(),
            Self::Unauthorized => UNAUTHORIZED.clone(),
            Self::PaymentRequired => PAYMENT_REQUIRED.clone(),
            Self::Forbidden => FORBIDDEN.clone(),
            Self::NotFound => NOT_FOUND.clone(),
            Self::MethodNotAllowed => METHOD_NOT_ALLOWED.clone(),
            Self::NotAcceptable => NOT_ACCEPTABLE.clone(),
            Self::ProxyAuthenticationRequired => PROXY_AUTHENTICATION_REQUIRED.clone(),
            Self::RequestTimeout => REQUEST_TIMEOUT.clone(),
            Self::Conflict => CONFLICT.clone(),
            Self::Gone => GONE.clone(),
            Self::LengthRequired => LENGTH_REQUIRED.clone(),
            Self::ConditionalRequestFailed => CONDITIONAL_REQUEST_FAILED.clone(),
            Self::RequestEntityTooLarge => REQUEST_ENTITY_TOO_LARGE.clone(),
            Self::RequestUriTooLong => REQUEST_URI_TOO_LONG.clone(),
            Self::UnsupportedMediaType => UNSUPPORTED_MEDIA_TYPE.clone(),
            Self::UnsupportedUriScheme => UNSUPPORTED_URI_SCHEME.clone(),
            Self::UnknownResourcePriority => UNKNOWN_RESOURCE_PRIORITY.clone(),
            Self::BadExtension => BAD_EXTENSION.clone(),
            Self::ExtensionRequired => EXTENSION_REQUIRED.clone(),
            Self::SessionIntervalTooSmall => SESSION_INTERVAL_TOO_SMALL.clone(),
            Self::IntervalTooBrief => INTERVAL_TOO_BRIEF.clone(),
            Self::BadLocationInformation => BAD_LOCATION_INFORMATION.clone(),
            Self::UseIdentityHeader => USE_IDENTITY_HEADER.clone(),
            Self::ProvideReferrerIdentity => PROVIDE_REFERRER_IDENTITY.clone(),
            Self::FlowFailed => FLOW_FAILED.clone(),
            Self::AnonymityDisallowed => ANONYMITY_DISALLOWED.clone(),
            Self::BadIdentityInfo => BAD_IDENTITY_INFO.clone(),
            Self::UnsupportedCertificate => UNSUPPORTED_CERTIFICATE.clone(),
            Self::InvalidIdentityHeader => INVALID_IDENTITY_HEADER.clone(),
            Self::FirstHopLacksOutboundSupport => FIRST_HOP_LACKS_OUTBOUND_SUPPORT.clone(),
            Self::MaxBreadthExceeded => MAX_BREADTH_EXCEEDED.clone(),
            Self::BadInfoPackage => BAD_INFO_PACKAGE.clone(),
            Self::ConsentNeeded => CONSENT_NEEDED.clone(),
            Self::TemporarilyUnavailable => TEMPORARILY_UNAVAILABLE.clone(),
            Self::CallOrTransactionDoesNotExist => CALL_OR_TRANSACTION_DOES_NOT_EXIST.clone(),
            Self::LoopDetected => LOOP_DETECTED.clone(),
            Self::TooManyHops => TOO_MANY_HOPS.clone(),
            Self::AddressIncomplete => ADDRESS_INCOMPLETE.clone(),
            Self::Ambiguous => AMBIGUOUS.clone(),
            Self::BusyHere => BUSY_HERE.clone(),
            Self::RequestTerminated => REQUEST_TERMINATED.clone(),
            Self::NotAcceptableHere => NOT_ACCEPTABLE_HERE.clone(),
            Self::BadEvent => BAD_EVENT.clone(),
            Self::RequestUpdated => REQUEST_UPDATED.clone(),
            Self::RequestPending => REQUEST_PENDING.clone(),
            Self::Undecipherable => UNDECIPHERABLE.clone(),
            Self::SecurityAgreementRequired => SECURITY_AGREEMENT_REQUIRED.clone(),
            Self::ServerInternalError => SERVER_INTERNAL_ERROR.clone(),
            Self::NotImplemented => NOT_IMPLEMENTED.clone(),
            Self::BadGateway => BAD_GATEWAY.clone(),
            Self::ServiceUnavailable => SERVICE_UNAVAILABLE.clone(),
            Self::ServerTimeout => SERVER_TIMEOUT.clone(),
            Self::VersionNotSupported => VERSION_NOT_SUPPORTED.clone(),
            Self::MessageTooLarge => MESSAGE_TOO_LARGE.clone(),
            Self::PushNotificationServiceNotSupported => {
                PUSH_NOTIFICATION_SERVICE_NOT_SUPPORTED.clone()
            }
            Self::PreconditionFailure => PRECONDITION_FAILURE.clone(),
            Self::BusyEverywhere => BUSY_EVERYWHERE.clone(),
            Self::Decline => DECLINE.clone(),
            Self::DoesNotExistAnywhere => DOES_NOT_EXIST_ANYWHERE.clone(),
            Self::NotAcceptableAnywhere => NOT_ACCEPTABLE_ANYWHERE.clone(),
            Self::Unwanted => UNWANTED.clone(),
            Self::Rejected => REJECTED.clone(),
        }
    }

    ///  Returns the class of the status code.
    pub fn class(&self) -> CodeClass {
        match self.as_u16() {
            100..=199 => CodeClass::Provisional,
            200..=299 => CodeClass::Success,
            300..=399 => CodeClass::Redirection,
            400..=499 => CodeClass::ClientError,
            500..=599 => CodeClass::ServerError,
            600..=699 => CodeClass::GlobalFailure,
            _ => unreachable!("StatusCode::class called on an invalid status code"),
        }
    }

    /// Converts a `StatusCode` into its numeric code.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Returns [`true`] if its status code is provisional (from `100` to
    /// `199`), and [`false`] otherwise.
    #[inline]
    pub fn is_provisional(&self) -> bool {
        matches!(self.class(), CodeClass::Provisional)
    }

    /// Returns [`true`]  if its status code is final (from `200` to `699` ),
    /// and [`false`] otherwise.
    #[inline]
    pub fn is_final(&self) -> bool {
        !self.is_provisional()
    }
}

impl TryFrom<&[u8]> for StatusCode {
    type Error = ();

    fn try_from(code: &[u8]) -> Result<Self, Self::Error> {
        Ok(match code {
            b"100" => Self::Trying,
            b"180" => Self::Ringing,
            b"181" => Self::CallIsBeingForwarded,
            b"182" => Self::Queued,
            b"183" => Self::SessionProgress,
            b"199" => Self::EarlyDialogTerminated,
            b"200" => Self::Ok,
            b"202" => Self::Accepted,
            b"204" => Self::NoNotification,
            b"300" => Self::MultipleChoices,
            b"301" => Self::MovedPermanently,
            b"302" => Self::MovedTemporarily,
            b"305" => Self::UseProxy,
            b"380" => Self::AlternativeService,
            b"400" => Self::BadRequest,
            b"401" => Self::Unauthorized,
            b"402" => Self::PaymentRequired,
            b"403" => Self::Forbidden,
            b"404" => Self::NotFound,
            b"405" => Self::MethodNotAllowed,
            b"406" => Self::NotAcceptable,
            b"407" => Self::ProxyAuthenticationRequired,
            b"408" => Self::RequestTimeout,
            b"409" => Self::Conflict,
            b"410" => Self::Gone,
            b"411" => Self::LengthRequired,
            b"412" => Self::ConditionalRequestFailed,
            b"413" => Self::RequestEntityTooLarge,
            b"414" => Self::RequestUriTooLong,
            b"415" => Self::UnsupportedMediaType,
            b"416" => Self::UnsupportedUriScheme,
            b"417" => Self::UnknownResourcePriority,
            b"420" => Self::BadExtension,
            b"421" => Self::ExtensionRequired,
            b"422" => Self::SessionIntervalTooSmall,
            b"423" => Self::IntervalTooBrief,
            b"424" => Self::BadLocationInformation,
            b"428" => Self::UseIdentityHeader,
            b"429" => Self::ProvideReferrerIdentity,
            b"430" => Self::FlowFailed,
            b"433" => Self::AnonymityDisallowed,
            b"436" => Self::BadIdentityInfo,
            b"437" => Self::UnsupportedCertificate,
            b"438" => Self::InvalidIdentityHeader,
            b"439" => Self::FirstHopLacksOutboundSupport,
            b"440" => Self::MaxBreadthExceeded,
            b"469" => Self::BadInfoPackage,
            b"470" => Self::ConsentNeeded,
            b"480" => Self::TemporarilyUnavailable,
            b"481" => Self::CallOrTransactionDoesNotExist,
            b"482" => Self::LoopDetected,
            b"483" => Self::TooManyHops,
            b"484" => Self::AddressIncomplete,
            b"485" => Self::Ambiguous,
            b"486" => Self::BusyHere,
            b"487" => Self::RequestTerminated,
            b"488" => Self::NotAcceptableHere,
            b"489" => Self::BadEvent,
            b"490" => Self::RequestUpdated,
            b"491" => Self::RequestPending,
            b"493" => Self::Undecipherable,
            b"494" => Self::SecurityAgreementRequired,
            b"500" => Self::ServerInternalError,
            b"501" => Self::NotImplemented,
            b"502" => Self::BadGateway,
            b"503" => Self::ServiceUnavailable,
            b"504" => Self::ServerTimeout,
            b"505" => Self::VersionNotSupported,
            b"513" => Self::MessageTooLarge,
            b"555" => Self::PushNotificationServiceNotSupported,
            b"580" => Self::PreconditionFailure,
            b"600" => Self::BusyEverywhere,
            b"603" => Self::Decline,
            b"604" => Self::DoesNotExistAnywhere,
            b"606" => Self::NotAcceptableAnywhere,
            b"607" => Self::Unwanted,
            b"608" => Self::Rejected,
            _ => return Err(()),
        })
    }
}
