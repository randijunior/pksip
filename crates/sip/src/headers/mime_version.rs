use std::str;

use scanner::Scanner;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `MIME-Version` SIP header.
///
/// Indicate what version of the `MIME` protocol was used to construct the message.
#[derive(Debug, PartialEq)]
pub struct MimeVersion<'a>(&'a str);

impl<'a> SipHeader<'a> for MimeVersion<'a> {
    const NAME: &'static str = "MIME-Version";

    fn parse(scanner: &mut Scanner<'a>) -> Result<MimeVersion<'a>> {
        let expires = scanner.scan_number_as_str();

        Ok(MimeVersion(expires))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"1.0";
        let mut scanner = Scanner::new(src);
        let mime_version = MimeVersion::parse(&mut scanner).unwrap();

        assert_eq!(mime_version.0, "1.0");
    }
}
