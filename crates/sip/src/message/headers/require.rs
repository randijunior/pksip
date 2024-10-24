use core::str;

use crate::{
    bytes::Bytes,
    macros::space,
    parser::{self, Result},
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Require<'a> {
    const NAME: &'static [u8] = b"Require";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tag = parser::parse_token(bytes);
        let mut tags = vec![tag];

        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let tag = parser::parse_token(bytes);
            tags.push(tag);
            space!(bytes);
        }

        Ok(Require(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel\r\n";
        let mut bytes = Bytes::new(src);
        let require = Require::parse(&mut bytes);
        let require = require.unwrap();

        assert_eq!(require, Require(vec!["100rel"]));
    }
}
