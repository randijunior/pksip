use std::str;

use reader::Reader;

use crate::macros::hdr_list;
use crate::parser::Result;
use crate::token::Token;

use crate::headers::SipHeader;

/// The `Require` SIP header.
///
/// Is used by `UACs` to tell `UASs` about options that the
/// `UAC` expects the `UAS` to support in order to process the
/// request.
#[derive(Debug, PartialEq, Eq)]
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Require<'a> {
    const NAME: &'static str = "Require";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = hdr_list!(reader => Token::parse(reader)?);

        Ok(Require(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel\r\n";
        let mut reader = Reader::new(src);
        let require = Require::parse(&mut reader);
        let require = require.unwrap();

        assert_eq!(require.0.get(0), Some(&"100rel"));
    }
}
