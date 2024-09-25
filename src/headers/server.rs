use core::str;

use crate::{byte_reader::ByteReader, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct Server<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Server<'a> {
    const NAME: &'static [u8] = b"Server";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let val = until_newline!(reader);
        let val = str::from_utf8(val)?;

        Ok(Server(val))
    }
}
