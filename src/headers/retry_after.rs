use crate::{byte_reader::ByteReader, macros::digits, parser::Result, uri::Params};

use super::SipHeaderParser;

pub struct RetryAfter<'a> {
    seconds: u32,
    param: Option<Params<'a>>,
    comment: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for RetryAfter<'a> {
    const NAME: &'a [u8] = b"Retry-After";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        todo!()
    }
}
