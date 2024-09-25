use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{read_while, space},
    parser::{is_token, Result},
};

use super::{OptionTag, SipHeaderParser};

pub struct Unsupported<'a>(Vec<OptionTag<'a>>);

impl<'a> SipHeaderParser<'a> for Unsupported<'a> {
    const NAME: &'static [u8] = b"Unsupported";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let tag = read_while!(reader, is_token);
        let tag = unsafe { str::from_utf8_unchecked(tag) };
        let mut tags = vec![OptionTag(tag)];

        while let Some(b',') = reader.peek() {
            let tag = read_while!(reader, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
            tags.push(OptionTag(tag));
            space!(reader);
        }

        Ok(Unsupported(tags))
    }
}
