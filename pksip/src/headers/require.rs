use std::{fmt, str};

use itertools::Itertools;

use crate::error::Result;
use crate::macros::hdr_list;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Require` SIP header.
///
/// Is used by `UACs` to tell `UASs` about options that the
/// `UAC` expects the `UAS` to support in order to process
/// the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeaderParse<'a> for Require<'a> {
    const NAME: &'static str = "Require";
    /*
     * Require  =  "Require" HCOLON option-tag *(COMMA option-tag)
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let tags = hdr_list!(parser => parser.parse_token()?);

        Ok(Require(tags))
    }
}

impl fmt::Display for Require<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Require::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel\r\n";
        let mut scanner = ParseCtx::new(src);
        let require = Require::parse(&mut scanner);
        let require = require.unwrap();

        assert_eq!(require.0.get(0), Some(&"100rel".into()));
    }
}
