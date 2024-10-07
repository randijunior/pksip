use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct CallId<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for CallId<'a> {
    const NAME: &'static [u8] = b"Call-ID";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"i");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let id = until_newline!(scanner);
        let id = str::from_utf8(id)?;

        Ok(CallId(id))
    }
}
