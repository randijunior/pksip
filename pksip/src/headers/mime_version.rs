use std::{fmt, str};

use crate::error::Result;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `MIME-Version` SIP header.
///
/// Indicate what version of the `MIME` protocol was used to
/// construct the message.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MimeVersion {
    major: u8,
    minor: u8,
}

impl<'a> SipHeaderParse<'a> for MimeVersion {
    const NAME: &'static str = "MIME-Version";
    /*
     * MIME-Version  =  "MIME-Version" HCOLON 1*DIGIT "." 1*DIGIT
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let (major, _, minor) = (parser.parse_num()?, parser.must_read(b'.')?, parser.parse_num()?);

        Ok(MimeVersion { major, minor })
    }
}

impl fmt::Display for MimeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}.{}", MimeVersion::NAME, self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"1.0";
        let mut scanner = ParseCtx::new(src);
        let mime_version = MimeVersion::parse(&mut scanner).unwrap();

        assert_eq!(mime_version.major, 1);
        assert_eq!(mime_version.minor, 0);
    }
}
