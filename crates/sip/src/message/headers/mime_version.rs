use core::str;

use crate::{
    bytes::Bytes,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use crate::headers::SipHeader;

/// Indicate what version of the `MIME` protocol was used to construct the message.
#[derive(Debug, PartialEq)]
pub struct MimeVersion(f32);

impl<'a> SipHeader<'a> for MimeVersion {
    const NAME: &'static str = "MIME-Version";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let digits = digits!(bytes);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(expires) => Ok(MimeVersion(expires)),
            Err(_) => return sip_parse_error!("invalid MIME-Version!"),
        }
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
