use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct Date<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Date<'a> {
    const NAME: &'static [u8] = b"Date";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let date = until_newline!(scanner);
        let date = str::from_utf8(date)?;

        Ok(Date(date))
    }
}
