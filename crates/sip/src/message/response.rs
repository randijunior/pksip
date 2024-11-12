use std::str;

use crate::{
    bytes::Bytes, headers::Headers, macros::{digits, newline, space, until_newline}, parser::{self, SipParser, SipParserError}
};

use super::SipStatusCode;

/// Represents an SIP Status-Line

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
        let mut bytes = Bytes::new(src);

        Self::parse(&mut bytes)
    }

    pub(crate) fn parse(
        bytes: &mut Bytes<'a>,
    ) -> Result<StatusLine<'a>, SipParserError> {
        SipParser::parse_sip_v2(bytes)?;

        space!(bytes);
        let digits = digits!(bytes);
        space!(bytes);

        let status_code = SipStatusCode::from(digits);
        let b = until_newline!(bytes);

        let rp = str::from_utf8(b)?;

        newline!(bytes);

        Ok(StatusLine::new(status_code, rp))
    }
}

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
