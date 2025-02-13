use core::fmt;
use std::str;

use itertools::Itertools;
use reader::Reader;

use crate::internal::ArcStr;
use crate::macros::hdr_list;
use crate::parser;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Content-Encoding` SIP header.
///
/// Indicates what decoding mechanisms must be applied to obtain the media-type
/// referenced by the Content-Type header field.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentEncoding(Vec<ArcStr>);

impl ContentEncoding {
    pub fn get(&self, index: usize) -> Option<&str> {
        self.0.get(index).map(|s| s.as_ref())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl SipHeader<'_> for ContentEncoding {
    const NAME: &'static str = "Content-Encoding";
    const SHORT_NAME: &'static str = "e";
    /*
     * Content-Encoding  =  ( "Content-Encoding" / "e" ) HCOLON
     *                      content-coding *(COMMA content-coding)
     */
    fn parse(reader: &mut Reader) -> Result<ContentEncoding> {
        let codings = hdr_list!(reader => parser::parse_token(reader)?.into());

        Ok(ContentEncoding(codings))
    }
}

impl fmt::Display for ContentEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
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
