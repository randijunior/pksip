//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use crate::headers::Headers;

use super::{SipMethod, Uri};

/// Represents an SIP Request-Line.
#[derive(Debug)]
pub struct RequestLine<'a> {
    pub method: SipMethod<'a>,
    pub uri: Uri<'a>,
}

#[derive(Debug)]
pub struct SipRequest<'a> {
    pub req_line: RequestLine<'a>,
    pub headers: Headers<'a>,
    pub body: Option<&'a [u8]>,
}

impl<'a> SipRequest<'a> {
    pub fn new(
        req_line: RequestLine<'a>,
        headers: Headers<'a>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            body,
            req_line,
            headers,
        }
    }

    pub fn request_line(&self) -> &RequestLine {
        &self.req_line
    }
}
