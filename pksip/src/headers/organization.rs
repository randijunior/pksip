use std::{fmt, str};

use crate::error::Result;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

use super::Header;

/// The `Organization` SIP header.
///
/// The name of the organization to which the SIP
/// element issuing the request or response belongs.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Organization<'a>(&'a str);

impl<'a> SipHeaderParse<'a> for Organization<'a> {
    const NAME: &'static str = "Organization";
    /*
     * Organization  =  "Organization" HCOLON [TEXT-UTF8-TRIM]
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let organization = parser.parse_header_value_as_str()?;

        Ok(Organization(organization))
    }
}

impl fmt::Display for Organization<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Organization::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Boxes by Bob\r\n";
        let mut scanner = ParseCtx::new(src);
        let org = Organization::parse(&mut scanner).unwrap();

        assert_eq!(org.0, "Boxes by Bob");
    }
}
