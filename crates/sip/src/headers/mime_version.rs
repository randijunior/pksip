use std::str;

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `MIME-Version` SIP header.
///
/// Indicate what version of the `MIME` protocol was used to construct the message.
#[derive(Debug, PartialEq, Eq)]
pub struct MimeVersion<'a>(&'a str);

impl<'a> SipHeader<'a> for MimeVersion<'a> {
    const NAME: &'static str = "MIME-Version";

    fn parse(reader: &mut Reader<'a>) -> Result<MimeVersion<'a>> {
        let expires = reader.read_number_as_str();

        Ok(MimeVersion(expires))
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

        assert_eq!(mime_version.0, "1.0");
    }
}
