use core::fmt;
use std::str;

use itertools::Itertools;

use crate::error::Result;

use crate::macros::hdr_list;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Content-Encoding` SIP header.
///
/// Indicates what decoding mechanisms must be applied to
/// obtain the media-type referenced by the Content-Type
/// header field.
///
/// # Examples
///
/// ```
/// # use pksip::headers::ContentEncoding;
/// let encoding = ContentEncoding::from(["gzip", "deflate"]);
///
/// assert_eq!(
///     "Content-Encoding: gzip, deflate",
///     encoding.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContentEncoding<'a>(Vec<&'a str>);

impl<'a> ContentEncoding<'a> {
    ///
    pub fn new() -> Self {
        todo!()
    }
    /// Get the content encoding at the specified index.
    pub fn get(&'a self, index: usize) -> Option<&'a str> {
        self.0.get(index).copied()
    }

    /// Return the number of content encodings.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParse<'a> for ContentEncoding<'a> {
    const NAME: &'static str = "Content-Encoding";
    const SHORT_NAME: &'static str = "e";
    /*
     * Content-Encoding  =  ( "Content-Encoding" / "e" ) HCOLON
     *                      content-coding *(COMMA content-coding)
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let codings = hdr_list!(parser => parser.parse_token()?);

        Ok(ContentEncoding(codings))
    }
}

impl fmt::Display for ContentEncoding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ContentEncoding::NAME, self.0.iter().format(", "))
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for ContentEncoding<'a> {
    fn from(value: [&'a str; N]) -> Self {
        Self(Vec::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"gzip\r\n";
        let mut scanner = ParseCtx::new(src);
        let encoding = ContentEncoding::parse(&mut scanner);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 1);
        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));

        let src = b"gzip, deflate\r\n";
        let mut scanner = ParseCtx::new(src);
        let encoding = ContentEncoding::parse(&mut scanner);
        let encoding = encoding.unwrap();

        assert!(encoding.len() == 2);
        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(encoding.get(0), Some("gzip"));
        assert_eq!(encoding.get(1), Some("deflate"));
    }
}
