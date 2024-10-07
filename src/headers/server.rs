use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct Server<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Server<'a> {
    const NAME: &'static [u8] = b"Server";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let val = until_newline!(scanner);
        let val = str::from_utf8(val)?;

        Ok(Server(val))
    }
}
