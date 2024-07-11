use crate::{
    msg::{SipStatusCode, StatusLine},
    reader::{InputReader, ReaderError},
    util::{is_digit, is_newline, is_space},
};

use std::str::{self};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl<'a> From<ReaderError<'a>> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line: {}, column: {}, kind: {:?}, input: '{}'",
                err.pos.line,
                err.pos.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

pub struct SipParser<'parser> {
    reader: InputReader<'parser>,
}

impl<'parser> SipParser<'parser> {
    pub fn new(i: &'parser [u8]) -> Self {
        SipParser {
            reader: InputReader::new(i),
        }
    }

    pub fn parse_status_line(&mut self) -> Result<StatusLine, SipParserError> {
        self.reader.tag(SIPV2)?;
        self.reader.read_while(is_space)?;

        let status_code = self.reader.read_while(is_digit)?;
        let status_code = SipStatusCode::from(status_code);

        self.reader.read_while(is_space)?;

        let rp = self.reader.read_until(is_newline)?;
        let rp = str::from_utf8(rp).map_err(|_| SipParserError {
            message: "Reason phrase is invalid utf8!".to_string(),
        })?;

        Ok(StatusLine::new(status_code, rp))
    }
}
