use core::str;

use crate::{
    macros::space,
    parser::{self, Result},
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Require<'a> {
    const NAME: &'static [u8] = b"Require";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tag = parser::parse_token(scanner);
        let mut tags = vec![tag];

        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);
            let tag = parser::parse_token(scanner);
            tags.push(tag);
            space!(scanner);
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
        let mut scanner = Scanner::new(src);
        let require = Require::parse(&mut scanner);
        let require = require.unwrap();

        assert_eq!(require, Require(vec!["100rel"]));
    }
}
