use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct Subject<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Subject<'a> {
    const NAME: &'static [u8] = b"Subject";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"s");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let val = until_newline!(scanner);
        let val = str::from_utf8(val)?;

        Ok(Subject(val))
    }
}
