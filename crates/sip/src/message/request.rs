//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use std::sync::Arc;

use crate::headers::{CSeq, CallId, Header, Headers, SipHeader};

use super::{SipMethod, SipUri, Uri};

/// Represents an SIP Request-Line.
#[derive(Debug)]
pub struct RequestLine {
    pub method: SipMethod,
    pub uri: Uri,
}

#[derive(Debug)]
pub struct SipRequest {
    pub req_line: RequestLine,
    pub headers: Headers,
    pub body: Option<Arc<[u8]>>,
}

impl SipRequest {
    pub fn new(
        req_line: RequestLine,
        headers: Headers,
        body: Option<&[u8]>,
    ) -> Self {
        Self {
            body: body.map(|b| b.into()),
            req_line,
            headers,
        }
    }

    pub fn method(&self) -> SipMethod {
        self.req_line.method
    }

    pub fn req_line(&self) -> &RequestLine {
        &self.req_line
    }
}
