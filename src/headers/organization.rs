use core::str;

use crate::{byte_reader::ByteReader, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct Organization<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Organization<'a> {
    const NAME: &'a [u8] = b"Organization";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let organization = until_newline!(reader);
        let organization = str::from_utf8(organization)?;

        Ok(Organization(organization))
    }
}
