//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use std::sync::Arc;

use crate::{
    headers::Headers,
    transport::RequestHeaders,
};

use super::{SipMethod, Uri};

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
    pub req_headers: Option<Box<RequestHeaders>>,
    pub body: Option<Arc<[u8]>>,
}

impl SipRequest {
    pub fn new(
        req_line: RequestLine,
        headers: Headers,
        body: Option<&[u8]>,
        req_hdrs: Option<Box<RequestHeaders>>,
    ) -> Self {
        Self {
            body: body.map(|b| b.into()),
            req_line,
            headers,
            req_headers: req_hdrs,
        }
    }

    pub fn method(&self) -> SipMethod {
        self.req_line.method
    }

    pub fn req_line(&self) -> &RequestLine {
        &self.req_line
    }
}
