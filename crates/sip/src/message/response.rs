use std::str;

use reader::{digits, newline, space, until_newline, Reader};

use crate::{
    headers::Headers,
    parser::{self, SipParser, SipParserError},
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
        let mut reader = Reader::new(src);

        Self::parse(&mut reader)
    }

    pub(crate) fn parse(
        reader: &mut Reader<'a>,
    ) -> Result<StatusLine<'a>, SipParserError> {
        SipParser::parse_sip_v2(reader)?;

        space!(reader);
        let digits = digits!(reader);
        space!(reader);

        let status_code = SipStatusCode::from(digits);
        let b = until_newline!(reader);

        let rp = str::from_utf8(b)?;

        newline!(reader);

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
