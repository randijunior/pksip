use crate::{
    msg::{SipStatusCode, StatusLine},
    reader::{InputReader, ParseError},
};

const SIPV2: &[u8]  = "2.0".as_bytes();

pub struct Parser<'parser> {
    reader: InputReader<'parser>,
}

impl<'parser> Parser<'parser> {
    pub fn new(i: &'parser [u8]) -> Self {
        Parser {
            reader: InputReader::new(i),
        }
    }
    pub fn parse_sip_version(&mut self) -> Result<(), ParseError> {
        self.reader.tag(b"SIP/")?;

        let version = self.reader.read_n(3)?;

        if version != SIPV2 {
            return Err(ParseError::Other("Invalid Sip Version!"));
        }

        Ok(())
    }

    pub fn parse_status_line(&mut self) -> Result<StatusLine, ParseError> {
        self.parse_sip_version()?;
        self.reader.read_while(InputReader::is_space)?;

        let status_code = self.reader.read_while(InputReader::is_digit)?;
        let status_code = SipStatusCode::from(status_code);

        self.reader.read_while(InputReader::is_space)?;

        let reason_phrase = self.reader.read_while(|c| c != 0x0D && c != 0x0A)?;
        let reason_phrase = std::str::from_utf8(reason_phrase).expect("Invalid utf-8");

        Ok(StatusLine::new(status_code, reason_phrase))
    }
}
