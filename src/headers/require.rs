use core::str;

use crate::{macros::read_while, parser::is_token};

use super::SipHeaderParser;

pub struct Require<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Require<'a> {
    const NAME: &'a [u8] = b"Require";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let tag = read_while!(reader, is_token);
        let tag = unsafe { str::from_utf8_unchecked(tag) };
        let mut tags = vec![];
        tags.push(tag);

        while let Some(b',') = reader.peek() {
            let tag = read_while!(reader, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
            tags.push(tag);
        }

        Ok(Require(tags))
    }
}
