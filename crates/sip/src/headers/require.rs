use std::str;

use crate::macros::parse_header_list;
use crate::token::Token;
use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Require` SIP header.
///
/// Is used by `UACs` to tell `UASs` about options that the
/// `UAC` expects the `UAS` to support in order to process the
/// request.
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Require<'a> {
    const NAME: &'static str = "Require";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tags = parse_header_list!(scanner => Token::parse(scanner));

        Ok(Require(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel\r\n";
        let mut scanner = Scanner::new(src);
        let require = Require::parse(&mut scanner);
        let require = require.unwrap();

        assert_eq!(require.0.get(0), Some(&"100rel"));
    }
}
