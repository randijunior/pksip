use std::str;

use reader::Reader;

use crate::macros::hdr_list;
use crate::{message::Token, parser::Result};

use crate::headers::SipHeader;

/// The `Supported` SIP header.
///
/// Enumerates all the extensions supported by the `UAC` or `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Supported<'a> {
    const NAME: &'static str = "Supported";
    const SHORT_NAME: Option<&'static str> = Some("k");

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = hdr_list!(reader => Token::parse(reader)?);

        Ok(Supported(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel, other\r\n";
        let mut reader = Reader::new(src);
        let supported = Supported::parse(&mut reader);
        let supported = supported.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(supported.0.get(0), Some(&"100rel"));
        assert_eq!(supported.0.get(1), Some(&"other"));
    }
}
