use core::str;

use crate::macros::parse_header_list;
use crate::{bytes::Bytes, parser::Result, token::Token};

use crate::headers::SipHeader;

/// The `Content-Encoding` SIP header.
///
/// Indicates what decoding mechanisms must be applied to obtain the media-type
/// referenced by the Content-Type header field.
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

    fn parse(bytes: &mut Bytes<'a>) -> Result<ContentEncoding<'a>> {
        let codings = parse_header_list!(bytes => Token::parse(bytes));

        Ok(ContentEncoding(codings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"gzip\r\n";
        let mut bytes = Bytes::new(src);
        let encoding = ContentEncoding::parse(&mut bytes);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 1);
        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));

        let src = b"gzip, deflate\r\n";
        let mut bytes = Bytes::new(src);
        let encoding = ContentEncoding::parse(&mut bytes);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 2);
        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));
        assert_eq!(encoding.get(1), Some("deflate"));
    }
}
