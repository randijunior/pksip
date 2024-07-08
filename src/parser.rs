use crate::{
    msg::{SipStatusCode, StatusLine},
    reader::{InputReader, ReaderError},
    util::{is_digit, is_space},
};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
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
        self.reader.tag(SIPV2)?;
        Ok(())
    }

    pub fn parse_status_line(&mut self) -> Result<StatusLine, SipParserError> {
        self.parse_sip_version()?;
        
        self.reader.read_while(is_space)?;

        let status_code = self.reader.read_while(is_digit)?;
        let status_code = SipStatusCode::from(status_code);

        self.reader.read_while(is_space)?;

        let rp = self
            .reader
            .read_while_utf8(|c| c != b'\r' && c != b'\n')?;

        Ok(StatusLine::new(status_code, rp))
    }
}
