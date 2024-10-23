use core::str;

use crate::{
    headers::Headers,
    macros::{digits, newline, space, until_newline},
    parser::{self, SipParserError},
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
    pub fn from_bytes(src: &'a [u8]) -> Result<StatusLine, SipParserError> {
        let mut scanner = Scanner::new(src);

        Self::parse(&mut scanner)
    }

    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<StatusLine<'a>, SipParserError> {
        parser::parse_sip_v2(scanner)?;

        space!(scanner);
        let digits = digits!(scanner);
        space!(scanner);

        let status_code = SipStatusCode::from(digits);
        let bytes = until_newline!(scanner);

        let rp = str::from_utf8(bytes)?;

        newline!(scanner);

        Ok(StatusLine::new(status_code, rp))
    }
}

#[derive(Debug)]
pub struct SipResponse<'a> {
    pub(crate) st_line: StatusLine<'a>,
    pub(crate) headers: Headers<'a>,
    pub(crate) body: Option<&'a [u8]>,
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
