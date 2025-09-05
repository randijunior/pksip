#![deny(missing_docs)]
//! SIP SipMessage types
//!
//! The module provide the [`SipMessage`] enum that can be an
//! [`SipMessage::Request`] or [`SipMessage::Response`] and represents a SIP
//! message.

use std::sync::Arc;

use crate::header::Headers;
use crate::parser::SIPV2;

mod auth;
mod code;
mod method;
mod param;
mod uri;

pub use auth::*;
pub use code::*;
pub use method::*;
pub use param::*;
pub use uri::*;

/// A SIP message.
///
/// It can be either a request from a client to a server,
/// or a response from a server to a client.
///
/// See [`Request`] and [`Response`] for more details.
pub enum SipMessage {
    /// An SIP Request.
    Request(Request),
    /// An SIP Response.
    Response(Response),
}

impl SipMessage {
    /// Returns `true` if this message is an [`Request`] message, and `false`
    /// otherwise.
    pub const fn is_request(&self) -> bool {
        matches!(self, SipMessage::Request(_))
    }

    /// Returns `true` if this message is an [`Response`] message, and `false`
    /// otherwise.
    pub const fn is_response(&self) -> bool {
        matches!(self, SipMessage::Response(_))
    }

    /// Returns a reference to the [`Request`] if this is a
    /// [`SipMessage::Request`] variant.
    pub fn request(&self) -> Option<&Request> {
        if let SipMessage::Request(request) = self {
            Some(request)
        } else {
            None
        }
    }

    /// Returns a reference to the [`Response`] if this is a
    /// [`SipMessage::Response`] variant.
    pub fn response(&self) -> Option<&Response> {
        if let SipMessage::Response(response) = self {
            Some(response)
        } else {
            None
        }
    }

    /// Returns a reference to the headers of the message.
    pub fn headers(&self) -> &Headers {
        match self {
            SipMessage::Request(req) => &req.headers,
            SipMessage::Response(res) => &res.headers,
        }
    }

    /// Returns a reference to the message body.
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            SipMessage::Request(request) => request.body.as_deref(),
            SipMessage::Response(response) => response.body.as_deref(),
        }
    }

    /// Returns a mutable reference to the headers of this [`SipMessage`].
    pub fn headers_mut(&mut self) -> &mut Headers {
        match self {
            SipMessage::Request(req) => &mut req.headers,
            SipMessage::Response(res) => &mut res.headers,
        }
    }

    /// Sets the body of the message. It can be `None` to remove the body.
    pub fn set_body(&mut self, body: Option<&[u8]>) {
        match self {
            SipMessage::Request(req) => {
                req.body = body.map(|b| b.into());
            }
            SipMessage::Response(res) => {
                res.body = body.map(|b| b.into());
            }
        }
    }

    /// Sets the headers of the message, replacing any existing headers.
    pub fn set_headers(&mut self, headers: Headers) {
        match self {
            SipMessage::Request(req) => {
                req.headers = headers;
            }
            SipMessage::Response(res) => {
                res.headers = headers;
            }
        }
    }
}

impl From<Request> for SipMessage {
    fn from(value: Request) -> Self {
        SipMessage::Request(value)
    }
}

impl From<Response> for SipMessage {
    fn from(value: Response) -> Self {
        SipMessage::Response(value)
    }
}

/// A parsed SIP Request.
///
/// SIP request represents a request from a client to a server.
pub struct Request {
    /// The Request-Line of the SIP message.
    pub req_line: RequestLine,
    /// All headers present in the SIP message.
    pub headers: Headers,
    /// The body of the SIP message, if present.
    pub body: Option<Arc<[u8]>>,
}

impl Request {
    /// Creates a new SIP `Request`.
    pub fn new(req_line: RequestLine) -> Self {
        Request {
            req_line,
            headers: Headers::new(),
            body: None,
        }
    }

    /// Creates a new `Request` with the given headers.
    #[inline]
    pub const fn with_headers(req_line: RequestLine, headers: Headers) -> Self {
        Self {
            req_line,
            headers,
            body: None,
        }
    }

    /// Returns the SIP method of the request.
    pub fn method(&self) -> SipMethod {
        self.req_line.method
    }
}

impl std::fmt::Display for RequestLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {SIPV2}\r\n", self.method, self.uri)
    }
}

/// Represents a SIP Request-Line.
///
/// The Request-Line contains the method and the Request-URI, which indicate the
/// target of the SIP request.
pub struct RequestLine {
    /// The SIP method associated with the request.
    pub method: SipMethod,
    /// The Request-URI indicating the target of the request.
    pub uri: Uri,
}

impl RequestLine {
    /// Creates a new `RequestLine` instance from the given [`SipMethod`] and
    /// [`Uri`].
    pub const fn new(method: SipMethod, uri: Uri) -> Self {
        Self { method, uri }
    }
}

/// A parsed SIP Response.
pub struct Response {
    /// The Status-Line of the SIP message.
    pub status_line: StatusLine,
    /// All headers present in the SIP message.
    pub headers: Headers,
    /// The body of the SIP message, if present.
    pub body: Option<Arc<[u8]>>,
}

impl Response {
    /// Creates a new SIP `Response` from a `Status-Line`, with empty headers
    /// and no body.
    pub fn new(status_line: StatusLine) -> Self {
        Self {
            status_line,
            headers: Default::default(),
            body: None,
        }
    }

    /// Returns the message response code.
    pub fn code(&self) -> StatusCode {
        self.status_line.code
    }

    /// Returns the reason.
    pub fn reason(&self) -> &str {
        &self.status_line.reason
    }

    /// Creates a new `Response` with the given `Status-Line` and headers,
    pub const fn with_headers(status_line: StatusLine, headers: Headers) -> Self {
        Self {
            status_line,
            headers,
            body: None,
        }
    }

    /// Creates a new `Response` with the given `Status-Line`, reason, and body.
    pub fn with_body(status_line: StatusLine, body: &[u8]) -> Self {
        Self {
            status_line,
            headers: Default::default(),
            body: Some(body.into()),
        }
    }

    /// Set the headers of the response, replacing any existing headers.
    pub fn set_headers(&mut self, headers: Headers) {
        self.headers = headers;
    }

    /// Appends headers from another collection to the current headers.
    pub fn append_headers(&mut self, other: &mut Headers) {
        self.headers.append(other);
    }
}

/// Represents a SIP Status-Line.
///
/// The Status-Line appears in SIP responses and includes a status code and a
/// `reason-phrase` explaining the result of the request.
pub struct StatusLine {
    /// The SIP status code associated with the response.
    pub code: StatusCode,
    /// The reason phrase explaining the status code.
    pub reason: Arc<str>,
}

impl std::fmt::Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code as i32, self.reason)
    }
}

impl StatusLine {
    /// Creates a new `StatusLine` instance from the given
    /// [`StatusCode`] and `reason-phrase`.
    pub fn new(code: StatusCode, reason: Arc<str>) -> Self {
        StatusLine { code, reason }
    }
}
