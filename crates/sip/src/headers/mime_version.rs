use std::str;

use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// The `MIME-Version` SIP header.
///
/// Indicate what version of the `MIME` protocol was used to construct the message.
#[derive(Debug, PartialEq)]
pub struct MimeVersion(f32);

impl<'a> SipHeader<'a> for MimeVersion {
    const NAME: &'static str = "MIME-Version";

    fn parse(bytes: &mut Bytes<'a>) -> Result<MimeVersion> {
        let expires = bytes.parse_num()?;

        Ok(MimeVersion(expires))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"1.0";
        let mut bytes = Bytes::new(src);
        let mime_version = MimeVersion::parse(&mut bytes).unwrap();

        assert_eq!(mime_version.0, 1.0);
    }
}
