use std::str;

use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Organization` SIP header.
///
/// The name of the organization to which the SIP
/// element issuing the request or response belongs.
pub struct Organization<'a>(&'a str);

impl<'a> SipHeader<'a> for Organization<'a> {
    const NAME: &'static str = "Organization";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let organization = Self::parse_as_str(scanner)?;

        Ok(Organization(organization))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Boxes by Bob\r\n";
        let mut scanner = Scanner::new(src);
        let org = Organization::parse(&mut scanner).unwrap();

        assert_eq!(org.0, "Boxes by Bob");
    }
}
