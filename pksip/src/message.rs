#![deny(missing_docs)]
//! SIP SipMessage types
//!
//! The module provide the [`SipMessage`] enum that can be an
//! [`SipMessage::Request`] or [`SipMessage::Response`] and represents a SIP
//! message.

use std::sync::Arc;

use crate::header::Headers;
use crate::parser::SIPV2;
use crate::ArcStr;

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

/// A SIP message as defined in [RFC 3261].
///
/// It can be either a request from a client to a server,
/// or a response from a server to a client.
///
/// See [`Request`] and [`Response`] for more details.
///
/// [RFC 3261]: https://datatracker.ietf.org/doc/html/rfc3261
pub enum SipMessage {
    /// An SIP Request.
    Request(Request),
    /// An SIP Response.
    Response(Response),
}

impl SipMessage {
    /// Returns `true` if this message is an [`Request`] message, and `false`
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Request;
    /// use pksip::message::RequestLine;
    /// use pksip::message::SipMessage;
    /// use pksip::message::SipMethod;
    ///
    /// let request = Request::new(RequestLine::new(
    ///     SipMethod::Options,
    ///     "sip:localhost".parse().unwrap(),
    /// ));
    /// let message: SipMessage = request.into();
    ///
    /// assert!(message.is_request());
    /// ```
    pub const fn is_request(&self) -> bool {
        matches!(self, SipMessage::Request(_))
    }

    /// Returns `true` if this message is an [`Response`] message, and `false`
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    /// use pksip::message::StatusLine;
    ///
    /// let response = Response::new(StatusLine::new(200, "OK"));
    /// let message: SipMessage = response.into();
    ///
    /// assert!(message.is_response());
    /// ```
    pub const fn is_response(&self) -> bool {
        matches!(self, SipMessage::Response(_))
    }

    /// Returns a reference to the [`Request`] if this is a
    /// [`SipMessage::Request`] variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Request;
    /// use pksip::message::RequestLine;
    /// use pksip::message::SipMessage;
    /// use pksip::message::SipMethod;
    ///
    /// let request = Request::new(RequestLine::new(
    ///     SipMethod::Options,
    ///     "sip:localhost".parse().unwrap(),
    /// ));
    /// let message: SipMessage = request.into();
    ///
    /// assert!(message.request().is_some());
    /// ```
    pub fn request(&self) -> Option<&Request> {
        if let SipMessage::Request(request) = self {
            Some(request)
        } else {
            None
        }
    }

    /// Returns a reference to the [`Response`] if this is a
    /// [`SipMessage::Response`] variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    ///
    /// let response = Response::new(StatusLine::new(200, "OK"));
    /// let message: SipMessage = response.into();
    ///
    /// assert!(message.response().is_some());
    /// ```
    pub fn response(&self) -> Option<&Response> {
        if let SipMessage::Response(response) = self {
            Some(response)
        } else {
            None
        }
    }

    /// Returns a reference to the headers of the message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::header::Expires;
    /// use pksip::header::Header;
    /// use pksip::header::Headers;
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    /// use pksip::message::StatusLine;
    ///
    /// let headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// let response = Response::with_headers(StatusLine::new(200, "OK"), headers);
    /// let message: SipMessage = response.into();
    ///
    /// assert_eq!(message.headers().len(), 1);
    /// ```
    pub fn headers(&self) -> &Headers {
        match self {
            SipMessage::Request(req) => &req.headers,
            SipMessage::Response(res) => &res.headers,
        }
    }

    /// Returns a reference to the message body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    ///
    /// let body = "Hello, SIP!".as_bytes();
    /// let response = Response::with_body(200, "OK", body);
    /// let message: SipMessage = response.into();
    ///
    /// assert_eq!(message.body(), Some(body));
    /// ```
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            SipMessage::Request(request) => request.body.as_deref(),
            SipMessage::Response(response) => response.body.as_deref(),
        }
    }

    /// Returns a mutable reference to the headers of this [`SipMessage`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::header::Expires;
    /// use pksip::header::Header;
    /// use pksip::header::Headers;
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    /// use pksip::message::StatusLine;
    ///
    /// let response = Response::with_headers(
    ///     StatusLine::new(200, "OK"),
    ///     Headers::from([Header::Expires(Expires::new(10))]),
    /// );
    /// // Convert the response into a SipMessage
    /// let mut message: SipMessage = response.into();
    ///
    /// assert_eq!(message.headers().len(), 1);
    ///
    /// // Add a new header
    /// message
    ///     .headers_mut()
    ///     .push(Header::Expires(Expires::new(20)));
    ///
    /// assert_eq!(message.headers().len(), 2);
    /// ```
    pub fn headers_mut(&mut self) -> &mut Headers {
        match self {
            SipMessage::Request(req) => &mut req.headers,
            SipMessage::Response(res) => &mut res.headers,
        }
    }

    /// Sets the body of the message. It can be `None` to
    /// remove the body.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    /// use pksip::message::StatusLine;
    ///
    /// let old_body = "Hello, SIP!".as_bytes();
    /// let new_body = "New body content".as_bytes();
    ///
    /// let response = Response::with_body(StatusLine::new(200, "OK"), old_body);
    /// let mut message: SipMessage = response.into();
    ///
    /// assert_eq!(message.body(), Some(old_body));
    ///
    /// // Set a new body
    /// message.set_body(Some(new_body));
    ///
    /// assert_eq!(message.body(), Some(new_body));
    /// ```
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

    /// Sets the headers of the message, replacing any
    /// existing headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::header::Expires;
    /// use pksip::header::Header;
    /// use pksip::header::Headers;
    /// use pksip::message::Response;
    /// use pksip::message::SipMessage;
    /// use pksip::message::StatusLine;
    ///
    /// let status_line = StatusLine::new(200, "OK");
    /// let response = Response::new(status_line);
    /// let mut message: SipMessage = response.into();
    ///
    /// assert_eq!(message.headers().len(), 0);
    ///
    /// // Set new headers
    /// let new_headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// message.set_headers(new_headers);
    ///
    /// assert_eq!(message.headers().len(), 1);
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::RequestLine;
    /// use pksip::message::SipMethod;
    ///
    /// let mut request = Request::new(RequestLine::new(
    ///     SipMethod::Options,
    ///     "sip:localhost".parse().unwrap(),
    /// ));
    /// ```
    pub fn new(req_line: RequestLine) -> Self {
        Request {
            req_line,
            headers: Headers::new(),
            body: None,
        }
    }

    /// Creates a new `Request` with the given headers.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::header::Expires;
    /// use pksip::header::Header;
    /// use pksip::header::Headers;
    /// use pksip::message::Request;
    /// use pksip::message::RequestLine;
    /// use pksip::message::SipMethod;
    ///
    /// let headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// let request = Request::with_headers(
    ///     RequestLine::new(SipMethod::Options, "sip:localhost".parse().unwrap()),
    ///     headers,
    /// );
    ///
    /// assert_eq!(request.headers.len(), 1);
    /// assert!(request.headers[0].is_expires());
    /// ```
    #[inline]
    pub const fn with_headers(req_line: RequestLine, headers: Headers) -> Self {
        Self {
            req_line,
            headers,
            body: None,
        }
    }

    /// Returns the SIP method of the request.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::Request;
    /// use pksip::message::SipMethod;
    /// use pksip::message::Uri;
    ///
    /// let request = Request::new(RequestLine::new(
    ///     SipMethod::Options,
    ///     "sip:localhost".parse().unwrap(),
    /// ));
    ///
    /// assert_eq!(request.method(), &SipMethod::Options);
    /// ```
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
    pub reason: ArcStr,
}

impl std::fmt::Display for StatusLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code as i32, self.reason)
    }
}

impl StatusLine {
    /// Creates a new `StatusLine` instance from the given
    /// [`StatusCode`] and `reason-phrase`.
    pub fn new(code: StatusCode, reason: ArcStr) -> Self {
        StatusLine { code, reason }
    }
}
