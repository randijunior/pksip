#![deny(missing_docs)]
//! SIP Message types
//!
//! The module provide the [`SipMsg`] enum that can be an [`SipMsg::Request`] or
//! [`SipMsg::Response`] and represents a SIP message.

use crate::headers::Headers;
use crate::parser::SIPV2;

pub mod auth;

mod code;
mod method;
mod params;
mod protocol;
mod uri;

pub use code::*;
pub use method::*;
pub use params::*;
pub use protocol::*;
pub use uri::*;

/// An SIP message, either Request or Response.
///
/// This enum can contain either an [`Request`] or an [`Response`], see their
/// respective documentation for more details.
pub enum SipMsg<'m> {
    /// An SIP Request.
    Request(Request<'m>),
    /// An SIP Response.
    Response(Response<'m>),
}

impl<'m> SipMsg<'m> {
    /// Returns [`true`] if this message is an [`Request`] message, and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::*;
    ///
    /// let uri = Uri::from_static("sip:alice@example.com").unwrap();
    /// let request = Request::new(SipMethod::Options, uri);
    /// let msg: SipMsg = request.into();
    ///
    /// assert!(msg.is_request());
    /// ```
    pub const fn is_request(&self) -> bool {
        matches!(self, SipMsg::Request(_))
    }

    /// Returns [`true`] if this message is an [`Response`] message, and [`false`]
    /// otherwise.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::*;
    ///
    /// let response = Response::new(StatusLine::new(200.into(), "OK"));
    /// let msg: SipMsg = response.into();
    ///
    /// assert!(msg.is_response());
    /// ```
    pub const fn is_response(&self) -> bool {
        matches!(self, SipMsg::Response(_))
    }

    /// Returns a reference to the [`Request`] if this is a [`SipMsg::Request`] variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::*;
    ///
    /// let uri = Uri::from_static("sip:alice@example.com").unwrap();
    /// let request = Request::new(SipMethod::Options, uri);
    /// let msg: SipMsg = request.into();
    ///
    /// assert!(msg.request().is_some());
    /// ```
    pub fn request(&self) -> Option<&'m Request> {
        if let SipMsg::Request(request) = self {
            Some(request)
        } else {
            None
        }
    }

    /// Returns a reference to the [`Response`] if this is a [`SipMsg::Response`] variant.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pksip::message::*;
    ///
    /// let response = Response::new(StatusLine::new(200.into(), "OK"));
    /// let msg: SipMsg = response.into();
    ///
    /// assert!(msg.response().is_some());
    /// ```
    pub fn response(&self) -> Option<&'m Response> {
        if let SipMsg::Response(response) = self {
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
    /// use pksip::headers::{Header, Headers, Expires};
    /// use pksip::message::{SipMsg, Response, StatusLine};
    ///
    /// let status_line = StatusLine::new(200.into(), "OK");
    /// let headers = Headers::from([Header::Expires(Expires::new(10))]);
    ///
    /// let response = Response::new_with_headers(status_line, headers);
    /// let msg: SipMsg = response.into();
    ///
    /// assert_eq!(msg.headers().len(), 1);
    /// ```
    pub fn headers(&self) -> &Headers<'m> {
        match self {
            SipMsg::Request(req) => &req.headers,
            SipMsg::Response(res) => &res.headers,
        }
    }

    /// Returns a reference to the message body.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::message::{SipMsg, Response};
    /// 
    /// let body = "Hello, SIP!".as_bytes();
    /// let response = Response::new_with_body(200, "OK", body);
    /// let msg: SipMsg = response.into();
    /// 
    /// assert_eq!(msg.body(), Some(body));
    /// ```
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            SipMsg::Request(request) => request.body.as_deref(),
            SipMsg::Response(response) => response.body.as_deref(),
        }
    }

    /// Returns a mutable reference to the headers of the message.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::headers::{Header, Headers, Expires};
    /// use pksip::message::{SipMsg, Response, StatusLine};
    /// 
    /// let status_line = StatusLine::new(200.into(), "OK");
    /// let headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// let response = Response::new_with_headers(status_line, headers);
    /// // Convert the response into a SipMsg
    /// let mut msg: SipMsg = response.into();
    /// 
    /// assert_eq!(msg.headers().len(), 1);
    /// 
    /// // Add a new header
    /// msg.headers_mut().push(Header::Expires(Expires::new(20)));
    /// 
    /// assert_eq!(msg.headers().len(), 2);
    /// ```
    pub fn headers_mut(&mut self) -> &mut Headers<'m> {
        match self {
            SipMsg::Request(req) => &mut req.headers,
            SipMsg::Response(res) => &mut res.headers,
        }
    }

    /// Sets the body of the message. It can be `None` to remove the body.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::message::{SipMsg, Response, StatusLine};
    /// 
    /// let old_body = "Hello, SIP!".as_bytes();
    /// let new_body = "New body content".as_bytes();
    /// 
    /// let response = Response::new_with_body(200, "OK", old_body);
    /// let mut msg: SipMsg = response.into();
    /// 
    /// assert_eq!(msg.body(), Some(old_body));
    /// 
    /// // Set a new body
    /// msg.set_body(Some(new_body));
    /// 
    /// assert_eq!(msg.body(), Some(new_body));
    /// ```
    pub fn set_body(&mut self, body: Option<&'m [u8]>) {
        match self {
            SipMsg::Request(req) => {
                req.body = body;
            }
            SipMsg::Response(res) => {
                res.body = body;
            }
        }
    }

    /// Sets the headers of the message, replacing any existing headers.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::headers::{Header, Headers, Expires};
    /// use pksip::message::{SipMsg, Response, StatusLine};
    /// 
    /// let status_line = StatusLine::new(200.into(), "OK");
    /// let response = Response::new(status_line);
    /// let mut msg: SipMsg = response.into();
    /// 
    /// assert_eq!(msg.headers().len(), 0);
    /// 
    /// // Set new headers
    /// let new_headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// msg.set_headers(new_headers);
    /// 
    /// assert_eq!(msg.headers().len(), 1);
    /// ```
    pub fn set_headers(&mut self, headers: Headers<'m>) {
        match self {
            SipMsg::Request(req) => {
                req.headers = headers;
            }
            SipMsg::Response(res) => {
                res.headers = headers;
            }
        }
    }
}

impl<'m> From<Request<'m>> for SipMsg<'m> {
    fn from(value: Request<'m>) -> Self {
        SipMsg::Request(value)
    }
}

impl<'m> From<Response<'m>> for SipMsg<'m> {
    fn from(value: Response<'m>) -> Self {
        SipMsg::Response(value)
    }
}

/// A parsed SIP Request.
///
/// SIP request represents a request from a client to a server.
pub struct Request<'r> {
    /// The Request-Line of the SIP message.
    pub req_line: RequestLine<'r>,
    /// All headers present in the SIP message.
    pub headers: Headers<'r>,
    /// The body of the SIP message, if present.
    pub body: Option<&'r[u8]>,
}

impl<'r> Request<'r> {
    /// Creates a new SIP `Request`.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::message::{Request, SipMethod, Uri};
    /// 
    /// let uri = Uri::from_static("sip:localhost").unwrap();
    /// let mut request = Request::new(SipMethod::Options, uri);
    /// ```
    pub fn new(method: SipMethod, uri: Uri<'r>) -> Self {
        Request {
            req_line: RequestLine { method, uri },
            headers: Default::default(),
            body: None,
        }
    }

    /// Creates a new `Request` with the given headers.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::message::{Request, SipMethod, Uri};
    /// use pksip::headers::{Header, Headers, Expires};
    /// 
    /// let uri = Uri::from_static("sip:localhost").unwrap();
    /// let headers = Headers::from([Header::Expires(Expires::new(10))]);
    /// let request = Request::new_with_headers(SipMethod::Options, uri, headers);
    /// 
    /// assert_eq!(request.headers.len(), 1);
    /// assert_eq!(request.headers[0].as_expires().unwrap().as_u32(), 10);
    /// ```
    #[inline]
    pub const fn new_with_headers(method: SipMethod, uri: Uri<'r>, headers: Headers<'r>) -> Self {
        Self {
            req_line: RequestLine { method, uri },
            headers,
            body: None,
        }
    }

    /// Returns the SIP method of the request.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use pksip::message::{Request, SipMethod, Uri};
    /// 
    /// let uri = Uri::from_static("sip:localhost").unwrap();
    /// let request = Request::new(SipMethod::Options, uri);
    /// 
    /// assert_eq!(request.method(), &SipMethod::Options);
    pub fn method(&self) -> &SipMethod {
        &self.req_line.method
    }

    /// Convert this [`Request`] into an owned version of itself.
    pub fn into_owned(self) -> Request<'static> {
        unimplemented!()
        /* 
        Request {
            req_line: RequestLine {
                method: self.req_line.method,
                uri: self.req_line.uri.into_owned(),
            },
            headers: self.headers.into_owned(),
            body: self.body.map(|b| Cow::Owned(b.into_owned())),
        }
        */
    }
}

impl std::fmt::Display for RequestLine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {SIPV2}\r\n", self.method, self.uri)
    }
}

/// Represents a SIP Request-Line.
///
/// The Request-Line contains the method and the Request-URI,
/// which indicate the target of the SIP request.
pub struct RequestLine<'a> {
    /// The SIP method associated with the request (e.g., INVITE, BYE).
    pub method: SipMethod,
    /// The Request-URI indicating the target of the request.
    pub uri: Uri<'a>,
}

/// A parsed SIP Response.
pub struct Response<'a> {
    /// The Status-Line of the SIP message.
    pub status_line: StatusLine<'a>,
    /// All headers present in the SIP message.
    pub headers: Headers<'a>,
    /// The body of the SIP message, if present.
    pub body: Option<&'a [u8]>,
}

impl<'a> Response<'a> {
    /// Creates a new SIP `Response` from a `Status-Line`,
    /// with empty headers and no body.
    pub fn new(status_line: StatusLine<'a>) -> Self {
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
        self.status_line.reason
    }

    /// Creates a new `Response` with the given `Status-Line` and headers,
    pub const fn new_with_headers(status_line: StatusLine<'a>, headers: Headers<'a>) -> Self {
        Self {
            status_line,
            headers,
            body: None,
        }
    }

    /// Creates a new `Response` with the given `Status-Line`, reason, and body.
    pub fn new_with_body(code: i32, reason: &'a str, body: &'a [u8]) -> Self {
        Self {
            status_line: StatusLine { code: code.into(), reason },
            headers: Default::default(),
            body: Some(body),
        }
    }

    /// Set the headers of the response, replacing any existing headers.
    pub fn set_headers(&mut self, headers: Headers<'a>) {
        self.headers = headers;
    }

    /// Appends headers from another collection to the current headers.
    pub fn append_headers(&mut self, other: &mut Headers<'a>) {
        self.headers.append(other);
    }
}

/// Represents a SIP Status-Line.
///
/// The Status-Line appears in SIP responses and includes a
/// status code and a reason phrase explaining the result
/// of the request.
pub struct StatusLine<'a> {
    /// The SIP status code associated with the response (e.g., 200, 404).
    pub code: StatusCode,
    /// The reason phrase explaining the status code (e.g., "OK", "Not Found").
    pub reason: &'a str,
}

impl std::fmt::Display for StatusLine<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code.into_i32(), self.reason)
    }
}

impl<'a> StatusLine<'a> {
    /// Creates a new `StatusLine` instance from the given [`StatusCode`] and reason.
    ///
    /// # Examples
    /// ```
    /// # use pksip::message::StatusLine;
    /// let status_line = StatusLine::new(200.into(), "OK");
    /// ```
    pub const fn new(code: StatusCode, reason: &'a str) -> Self {
        StatusLine { code, reason }
    }
}
