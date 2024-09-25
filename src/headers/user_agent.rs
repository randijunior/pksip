use core::str;

use crate::{byte_reader::ByteReader, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct UserAgent<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for UserAgent<'a> {
    const NAME: &'static [u8] = b"User-Agent";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let val = until_newline!(reader);
        let val = str::from_utf8(val)?;

        Ok(UserAgent(val))
    }
}
