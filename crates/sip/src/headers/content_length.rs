use core::fmt;
use std::str;

use reader::{space, Reader};

use crate::parser::{parse_token, Result};

use crate::headers::SipHeader;

/// The `Content-Length` SIP header.
///
/// Indicates the size of the `message-body`.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentLength(pub u32);

impl ContentLength {
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut reader = Reader::new(bytes);
        let name = parse_token(&mut reader)?;
        assert_eq!(
            name,
            ContentLength::NAME,
            "The sip header name is not {}",
            ContentLength::NAME
        );

        reader.next();

        space!(reader);
        ContentLength::parse(&mut reader)
    }
}

impl<'a> SipHeader<'a> for ContentLength {
    const NAME: &'static str = "Content-Length";
    const SHORT_NAME: &'static str = "l";
    /*
     * Content-Length  =  ( "Content-Length" / "l" ) HCOLON 1*DIGIT
     */
    fn parse(reader: &mut Reader<'a>) -> Result<ContentLength> {
        let l = reader.read_u32()?;

        Ok(ContentLength(l))
    }
}

impl fmt::Display for ContentLength {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"349\r\n";
        let mut reader = Reader::new(src);
        let length = ContentLength::parse(&mut reader);
        let length = length.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(length.0, 349)
    }
}
