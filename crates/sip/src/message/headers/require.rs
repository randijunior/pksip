use core::str;

use crate::{
    bytes::Bytes,
    macros::space,
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// Is used by `UACs` to tell `UASs` about options that the
/// `UAC` expects the `UAS` to support in order to process the
/// request.
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Require<'a> {
    const NAME: &'static str = "Require";

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

        assert_eq!(require.0.get(0), Some(&"100rel"));
    }
}
