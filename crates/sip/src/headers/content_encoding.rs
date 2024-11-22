use std::str;

use reader::Reader;

use crate::macros::parse_header_list;
use crate::{parser::Result, token::Token};

use crate::headers::SipHeader;

/// The `Content-Encoding` SIP header.
///
/// Indicates what decoding mechanisms must be applied to obtain the media-type
/// referenced by the Content-Type header field.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentEncoding<'a>(Vec<&'a str>);

impl<'a> ContentEncoding<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.0.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeader<'a> for ContentEncoding<'a> {
    const NAME: &'static str = "Content-Encoding";
    const SHORT_NAME: Option<&'static str> = Some("e");

    fn parse(reader: &mut Reader<'a>) -> Result<ContentEncoding<'a>> {
        let codings = parse_header_list!(reader => Token::parse(reader));

        Ok(ContentEncoding(codings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"gzip\r\n";
        let mut reader = Reader::new(src);
        let encoding = ContentEncoding::parse(&mut reader);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 1);
        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));

        let src = b"gzip, deflate\r\n";
        let mut reader = Reader::new(src);
        let encoding = ContentEncoding::parse(&mut reader);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 2);
        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));
        assert_eq!(encoding.get(1), Some("deflate"));
    }
}
