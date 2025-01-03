use std::{
    fmt,
    io::{self},
    str,
};

use crate::{headers::Headers, parser::SIPV2};

use super::StatusCode;

/// Represents an SIP Status-Line.
#[derive(Debug)]
pub struct StatusLine<'sl> {
    // Status Code
    pub code: StatusCode,
    // Reason String
    pub rphrase: &'sl str,
}

impl fmt::Display for StatusLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code.as_str(), self.rphrase)
    }
}

impl<'sl> StatusLine<'sl> {
    pub fn new(st: StatusCode, rp: &'sl str) -> Self {
        StatusLine {
            code: st,
            rphrase: rp,
        }
    }
}

#[derive(Debug)]
pub struct SipResponse<'a> {
    pub st_line: StatusLine<'a>,
    pub headers: Headers<'a>,
    pub body: Option<&'a [u8]>,
}

impl<'a> SipResponse<'a> {
    pub fn new(
        st_line: StatusLine<'a>,
        headers: Headers<'a>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            body,
            st_line,
            headers,
        }
    }
}
