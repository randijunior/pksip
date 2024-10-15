use core::str;

use crate::{
    scanner::Scanner,
    macros::{read_while, space},
    parser::{is_token, Result},
};

use super::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Supported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Supported<'a> {
    const NAME: &'static [u8] = b"Supported";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"k");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tag = read_while!(scanner, is_token);
        let tag = unsafe { str::from_utf8_unchecked(tag) };
        let mut tags = vec![tag];

        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);
            let tag = read_while!(scanner, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
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
