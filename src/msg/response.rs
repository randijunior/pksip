use crate::{
    headers::SipHeaders,
    parser::{SipParser, SipParserError},
    scanner::Scanner,
};

use super::SipStatusCode;

/// This struct represent SIP status line
#[derive(Debug, PartialEq, Eq)]
pub struct StatusLine<'sl> {
    // Status Code
    pub(crate) status_code: SipStatusCode,
    // Reason String
    pub(crate) reason_phrase: &'sl str,
}

impl<'sl> StatusLine<'sl> {
    pub fn new(st: SipStatusCode, rp: &'sl str) -> Self {
        StatusLine {
            status_code: st,
            reason_phrase: rp,
        }
    }
}

impl<'a> StatusLine<'a> {
    pub fn from_bytes(src: &[u8]) -> Result<StatusLine, SipParserError> {
        let mut scanner = Scanner::new(src);

        SipParser::parse_status_line(&mut scanner)
    }
}

#[derive(Debug)]
pub struct SipResponse<'a> {
    pub(crate) st_line: StatusLine<'a>,
    pub(crate) headers: SipHeaders<'a>,
    pub(crate) body: Option<&'a [u8]>,
}

impl<'a> SipResponse<'a> {
    pub fn new(
        st_line: StatusLine<'a>,
        headers: SipHeaders<'a>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            body,
            st_line,
            headers,
        }
    }
}
