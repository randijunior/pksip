use core::str;

use crate::{
    macros::space,
    parser::{self, Result},
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Supported<'a> {
    const NAME: &'static [u8] = b"Supported";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"k");

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
        assert_eq!(supported, Supported(vec!["100rel", "other"]));
    }
}
