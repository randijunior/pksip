use crate::{byte_reader::ByteReader, macros::until_newline, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct CallId<'a> {
    id: &'a str,
}

impl<'a> SipHeaderParser<'a> for CallId<'a> {
    const NAME: &'a [u8] = b"Call-ID";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"i");

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let id = until_newline!(reader);
        let id = str::from_utf8(id)?;

        Ok(CallId { id })
    }
}
