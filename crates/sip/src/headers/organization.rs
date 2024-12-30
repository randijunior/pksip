use std::{fmt, str};

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Organization` SIP header.
///
/// The name of the organization to which the SIP
/// element issuing the request or response belongs.
#[derive(Debug, PartialEq, Eq)]
pub struct Organization<'a>(&'a str);

impl<'a> SipHeader<'a> for Organization<'a> {
    const NAME: &'static str = "Organization";
    /*
     * Organization  =  "Organization" HCOLON [TEXT-UTF8-TRIM]
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let organization = Self::parse_as_str(reader)?;

        Ok(Organization(organization))
    }
}

impl fmt::Display for Organization<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Boxes by Bob\r\n";
        let mut reader = Reader::new(src);
        let org = Organization::parse(&mut reader).unwrap();

        assert_eq!(org.0, "Boxes by Bob");
    }
}
