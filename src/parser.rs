use crate::{
    msg::{SipStatusCode, StatusLine},
    reader::{self, InputReader, ReaderError},
    util::{is_digit, is_newline, is_space},
};

use std::str;

const SIPV2: &[u8] = "2.0".as_bytes();

#[derive(Debug)]
pub enum SipParserError {
    InvalidStatusLine,
    ReaderError(ReaderError),
    // Outros erros específicos do Parser
}

impl From<ReaderError> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError::ReaderError(err)
    }
}

#[macro_export]
macro_rules! sip_parser_error {
    ($value:expr) => {
        match $value {
            0 => Ok(()),
            _ => Err(crate::util::Error::from($value)),
        }
        
    };
}

pub struct SipParser<'parser> {
    pub reader: InputReader<'parser>,
}

impl<'parser> SipParser<'parser> {
    pub fn new(i: &'parser [u8]) -> Self {
        SipParser {
            reader: InputReader::new(i),
        }
    }
    pub fn parse_sip_version(&mut self) -> Result<(), SipParserError> {
        self.reader.prefix(b"SIP")?;

        if self.reader.read()? != &b'/' || self.reader.read_n(3)? != SIPV2 {
            return Err(SipParserError::InvalidStatusLine);
        }

        Ok(())
    }

    pub fn parse_status_line(&mut self) -> Result<StatusLine, SipParserError> {
        self.parse_sip_version()?;
        
        let status_code = {
            self.reader.read_while(is_space)?;
            let bytes = self.reader.read_while(is_digit)?;

            SipStatusCode::from(bytes)
        };
        let reason_phrase = {
            self.reader.read_while(is_space)?;
            let bytes = self.reader.read_until_and_consume(is_newline)?;

            str::from_utf8(bytes).map_err(|_| ReaderError {
                err: crate::reader::ErrorKind::OutOfInput,
                pos: None,
            })?
        };

        Ok(StatusLine::new(status_code, reason_phrase))
    }
}
