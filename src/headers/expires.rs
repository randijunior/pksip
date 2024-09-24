use std::str;

use crate::{
    byte_reader::ByteReader,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

pub struct Expires(i32);

impl<'a> SipHeaderParser<'a> for Expires {
    const NAME: &'a [u8] = b"Expires";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        match str::from_utf8(digits)?.parse() {
            Ok(expires) => Ok(Expires(expires)),
            Err(_) => return sip_parse_error!("invalid Expires!"),
        }
    }
}
