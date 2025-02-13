use std::{
    fmt,
    io::{self},
    str,
    sync::Arc,
};

use crate::{headers::Headers, internal::ArcStr, parser::SIPV2};

use super::StatusCode;

/// Represents an SIP Status-Line.
#[derive(Debug)]
pub struct StatusLine {
    // Status Code
    pub code: StatusCode,
    // Reason String
    pub rphrase: ArcStr,
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code.as_str(), self.rphrase)
    }
}

impl StatusLine {
    pub fn new(st: StatusCode, rp: &str) -> Self {
        StatusLine {
            code: st,
            rphrase: rp.into(),
        }
    }
}

#[derive(Debug)]
pub struct SipResponse {
    pub st_line: StatusLine,
    pub headers: Headers,
    pub body: Option<Arc<[u8]>>,
}

impl SipResponse {
    pub fn new(
        st_line: StatusLine,
        headers: Headers,
        body: Option<&[u8]>,
    ) -> Self {
        Self {
            body: body.map(|b| b.into()),
            st_line,
            headers,
        }
    }
}
