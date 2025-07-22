#![deny(missing_docs)]
//! SIP Message types
//!
//! The module provide the [`SipMsg`] enum that can be an [`SipMsg::Request`] or
//! [`SipMsg::Response`] and represents a SIP message.

use crate::headers::Headers;
use crate::parser::SIPV2;

use std::fmt::Debug;

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

#[derive(Debug)]
/// An SIP message, either Request or Response.
///
/// This enum can contain either an [`Request`] or an [`Response`], see their
/// respective documentation for more details.
#[derive(PartialEq)]
pub enum SipMsg<'a> {
    /// An SIP Request.
    Request(Request<'a>),
    /// An SIP Response.
    Response(Response<'a>),
}

impl<'a> SipMsg<'a> {
    /// Returns [`true`] if this message is an [`Request` message], and [`false`]
    /// otherwise.
    ///
    /// [`Request` message]: SipMsg::Request
    pub const fn is_request(&self) -> bool {
        matches!(self, SipMsg::Request(_))
    }

    /// Returns [`true`] if this message is an [`Response` message], and [`false`]
    /// otherwise.
    ///
    /// [`Response` message]: SipMsg::Response
    pub const fn is_response(&self) -> bool {
        matches!(self, SipMsg::Response(_))
    }

    /// Returns a reference to the [`Request`] if this is a [`SipMsg::Request`] variant.
    pub fn request(&self) -> Option<&'a Request> {
        if let SipMsg::Request(request) = self {
            Some(request)
        } else {
            None
        }
    }

    /// Returns a reference to the [`Response`] if this is a [`SipMsg::Response`] variant.
    pub fn response(&self) -> Option<&'a Response> {
        if let SipMsg::Response(response) = self {
            Some(response)
        } else {
            None
        }
    }

    /// Returns a reference to the headers of the message.
    pub fn headers(&self) -> &Headers<'a> {
        match self {
            SipMsg::Request(req) => &req.headers,
            SipMsg::Response(res) => &res.headers,
        }
    }

    /// Returns a reference to the message body.
    pub fn body(&self) -> Option<&[u8]> {
        match self {
            SipMsg::Request(request) => request.body,
            SipMsg::Response(response) => response.body,
        }
    }

    /// Returns a mutable reference to the headers of the message.
    pub fn headers_mut(&mut self) -> &mut Headers<'a> {
        match self {
            SipMsg::Request(req) => &mut req.headers,
            SipMsg::Response(res) => &mut res.headers,
        }
    }

    /// Sets the body of the message. It can be `None` to remove the body.
    pub fn set_body(&mut self, body: Option<&'a [u8]>) {
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
    pub fn set_headers(&mut self, headers: Headers<'a>) {
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

#[derive(Debug, PartialEq)]
/// A parsed SIP Request.
///
/// SIP request represents a request from a client to a server.
pub struct Request<'a> {
    /// The Request-Line of the SIP message.
    pub req_line: RequestLine<'a>,
    /// All headers present in the SIP message.
    pub headers: Headers<'a>,
    /// The body of the SIP message, if present.
    pub body: Option<&'a [u8]>,
}

impl<'a> Request<'a> {
    /// Returns the SIP method of the request.
    pub fn method(&self) -> &SipMethod {
        &self.req_line.method
    }

    /// Convert
    pub fn into_owned(self) -> Request<'static> {
        todo!();
        /*
        Request {
            req_line: RequestLine {
                method: self.req_line.method,
                uri: self.req_line.uri.into_owned(),
            },
            headers: self.headers.into_owned(),
            body: self.body.map(|b| Box::leak(b.to_vec().into_boxed_slice())),
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
#[derive(Debug, PartialEq)]
pub struct RequestLine<'a> {
    /// The SIP method associated with the request (e.g., INVITE, BYE).
    pub method: SipMethod,
    /// The Request-URI indicating the target of the request.
    pub uri: Uri<'a>,
}

#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
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
