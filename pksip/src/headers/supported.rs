use std::{fmt, str};

use itertools::Itertools;

use crate::error::Result;
use crate::macros::hdr_list;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Supported` SIP header.
///
/// Enumerates all the extensions supported by the `UAC` or
/// `UAS`.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> Supported<'a> {
    /// Add a new tag to the list of supported tags.
    pub fn add_tag(&mut self, tag: &'a str) {
        self.0.push(tag);
    }
}

impl<'a> SipHeaderParse<'a> for Supported<'a> {
    const NAME: &'static str = "Supported";
    const SHORT_NAME: &'static str = "k";
    /*
     * Supported  =  ( "Supported" / "k" ) HCOLON
     *               [option-tag *(COMMA option-tag)]
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let tags = hdr_list!(parser => parser.parse_token()?);

        Ok(Supported(tags))
    }
}

impl fmt::Display for Supported<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Supported::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel, other\r\n";
        let mut scanner = ParseCtx::new(src);
        let supported = Supported::parse(&mut scanner);
        let supported = supported.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(supported.0.get(0), Some(&"100rel".into()));
        assert_eq!(supported.0.get(1), Some(&"other".into()));
    }
}
