use core::str;

use crate::{byte_reader::ByteReader, macros::read_while, parser::{is_token, Result}};

use super::SipHeaderParser;

pub struct Priority<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Priority<'a> {
    const NAME: &'a [u8] = b"Priority";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let priority = read_while!(reader, is_token);
        let priority = unsafe { str::from_utf8_unchecked(priority) };

        Ok(Priority(priority))
    }
}