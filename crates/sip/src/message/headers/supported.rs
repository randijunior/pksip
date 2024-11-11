use std::str;

use crate::macros::parse_header_list;
use crate::{bytes::Bytes, parser::Result, token::Token};

use crate::headers::SipHeader;

/// The `Supported` SIP header.
///
/// Enumerates all the extensions supported by the `UAC` or `UAS`.
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Supported<'a> {
    const NAME: &'static str = "Supported";
    const SHORT_NAME: Option<&'static str> = Some("k");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tags = parse_header_list!(bytes => Token::parse(bytes));

        Ok(Supported(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel, other\r\n";
        let mut bytes = Bytes::new(src);
        let supported = Supported::parse(&mut bytes);
        let supported = supported.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(supported.0.get(0), Some(&"100rel"));
        assert_eq!(supported.0.get(1), Some(&"other"));
    }
}
