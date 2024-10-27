use core::str;

use crate::{
    bytes::Bytes,
    macros::{read_while, space},
    parser::{self, is_token, Result},
};

use crate::headers::SipHeaderParser;

pub struct ContentEncoding<'a>(Vec<&'a str>);

impl<'a> ContentEncoding<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.0.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for ContentEncoding<'a> {
    const NAME: &'static [u8] = b"Content-Encoding";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"e");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut codings: Vec<&'a str> = Vec::new();
        let coding = parser::parse_token(bytes);
        codings.push(coding);

        space!(bytes);
        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let coding = parser::parse_token(bytes);
            codings.push(coding);
        }

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
        let c_enconding = ContentEncoding::parse(&mut bytes).unwrap();

        assert!(c_enconding.len() == 1);
        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_enconding.get(0), Some("gzip"));

        let src = b"gzip, deflate\r\n";
        let mut bytes = Bytes::new(src);
        let c_enconding = ContentEncoding::parse(&mut bytes).unwrap();

        assert!(c_enconding.len() == 2);
        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_enconding.get(0), Some("gzip"));
        assert_eq!(c_enconding.get(1), Some("deflate"));
    }
}
