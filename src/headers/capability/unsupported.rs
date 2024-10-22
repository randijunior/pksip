use core::str;

use crate::{
    macros::{read_while, space},
    parser::{self, is_token, Result},
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Unsupported<'a> {
    const NAME: &'static [u8] = b"Unsupported";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tag = parser::parse_token(scanner);
        let mut tags = vec![tag];

        while let Some(b',') = scanner.peek() {
            let tag = read_while!(scanner, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
            tags.push(tag);
            space!(scanner);
        }

        Ok(Unsupported(tags))
    }
}
