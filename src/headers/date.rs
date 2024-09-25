use crate::{byte_reader::ByteReader, macros::until_newline, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct Date<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Date<'a> {
    const NAME: &'static [u8] = b"Date";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let date = until_newline!(reader);
        let date = str::from_utf8(date)?;

        Ok(Date(date))
    }
}
