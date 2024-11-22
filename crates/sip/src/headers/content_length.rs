use std::str;

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Content-Length` SIP header.
///
/// Indicates the size of the `message-body`.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentLength(u32);

impl ContentLength {
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }
}

impl<'a> SipHeader<'a> for ContentLength {
    const NAME: &'static str = "Content-Length";
    const SHORT_NAME: Option<&'static str> = Some("l");

    fn parse(reader: &mut Reader<'a>) -> Result<ContentLength> {
        let l = reader.read_num()?;

        Ok(ContentLength(l))
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
