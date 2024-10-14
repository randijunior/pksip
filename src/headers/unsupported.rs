use core::str;

use crate::{
    scanner::Scanner,
    macros::{read_while, space},
    parser::{is_token, Result},
};

use super::{SipHeaderParser};

pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Unsupported<'a> {
    const NAME: &'static [u8] = b"Unsupported";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tag = read_while!(scanner, is_token);
        let tag = unsafe { str::from_utf8_unchecked(tag) };
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
