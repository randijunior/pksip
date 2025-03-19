//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use std::sync::Arc;

use crate::{
    headers::{CSeq, Headers},
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
    pub fn method(&self) -> &SipMethod {
        &self.req_line.method
    }

    pub fn req_line(&self) -> &RequestLine {
        &self.req_line
    }

    pub fn cseq(&self) -> Option<&CSeq> {
        self.req_headers.as_ref().map(|req| &req.cseq)
    }
}
