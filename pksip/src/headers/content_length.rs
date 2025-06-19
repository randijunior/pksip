use core::fmt;
use std::str;

use crate::error::Result;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Content-Length` SIP header.
///
/// Indicates the size, in bytes, of the `message-body`.
///
/// Both the long (`Content-Length`) and short (`l`) header names are supported.
///
/// # Examples
/// ```
/// # use pksip::headers::ContentLength;
/// let c_len = ContentLength::new(3600);
///
/// assert_eq!(
///     "Content-Length: 3600",
///     c_len.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct ContentLength(u32);

impl ContentLength {
    /// Creates a new `ContentLength` from a `u32`.
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }

    /// Returns the internal content length value.
    pub fn clen(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeaderParse<'a> for ContentLength {
    const NAME: &'static str = "Content-Length";
    const SHORT_NAME: &'static str = "l";
    /*
     * Content-Length  =  ( "Content-Length" / "l" ) HCOLON 1*DIGIT
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<ContentLength> {
        let l = parser.parse_u32()?;

        Ok(ContentLength(l))
    }
}

impl fmt::Display for ContentLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", ContentLength::NAME, self.0)
    }
}

impl Default for ContentLength {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"349\r\n";
        let mut scanner = ParseCtx::new(src);
        let length = ContentLength::parse(&mut scanner);
        let length = length.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(length.0, 349)
    }
}
