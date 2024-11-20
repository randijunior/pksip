use std::str;

use scanner::Scanner;

use crate::macros::parse_header_list;
use crate::{parser::Result, token::Token};

use crate::headers::SipHeader;

/// The `Supported` SIP header.
///
/// Enumerates all the extensions supported by the `UAC` or `UAS`.
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Supported<'a> {
    const NAME: &'static str = "Supported";
    const SHORT_NAME: Option<&'static str> = Some("k");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tags = parse_header_list!(scanner => Token::parse(scanner));

        Ok(Supported(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel, other\r\n";
        let mut scanner = Scanner::new(src);
        let supported = Supported::parse(&mut scanner);
        let supported = supported.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(supported.0.get(0), Some(&"100rel"));
        assert_eq!(supported.0.get(1), Some(&"other"));
    }
}
