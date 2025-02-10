use std::{fmt, str};

use reader::Reader;

use crate::internal::ArcStr;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `MIME-Version` SIP header.
///
/// Indicate what version of the `MIME` protocol was used to construct the message.
#[derive(Debug, PartialEq, Eq)]
pub struct MimeVersion(ArcStr);

impl SipHeader<'_> for MimeVersion {
    const NAME: &'static str = "MIME-Version";
    /*
     * MIME-Version  =  "MIME-Version" HCOLON 1*DIGIT "." 1*DIGIT
     */
    fn parse(reader: &mut Reader) -> Result<MimeVersion> {
        let expires = reader.read_number_as_str();

        Ok(MimeVersion(expires.into()))
    }
}

impl fmt::Display for MimeVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"1.0";
        let mut reader = Reader::new(src);
        let mime_version = MimeVersion::parse(&mut reader).unwrap();

        assert_eq!(mime_version.0, "1.0".into());
    }
}
