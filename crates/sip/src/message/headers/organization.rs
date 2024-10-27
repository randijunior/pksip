use core::str;

use crate::{bytes::Bytes, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;

pub struct Organization<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Organization<'a> {
    const NAME: &'static [u8] = b"Organization";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let organization = until_newline!(bytes);
        let organization = str::from_utf8(organization)?;

        Ok(Organization(organization))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Boxes by Bob\r\n";
        let mut bytes = Bytes::new(src);
        let org = Organization::parse(&mut bytes).unwrap();

        assert_eq!(org.0, "Boxes by Bob");
    }
}
