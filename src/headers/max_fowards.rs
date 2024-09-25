use crate::{
    byte_reader::ByteReader,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

use std::str;

pub struct MaxForwards(u32);

impl<'a> SipHeaderParser<'a> for MaxForwards {
    const NAME: &'static [u8] = b"Max-Forwards";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(digits) => Ok(MaxForwards(digits)),
            Err(_) => sip_parse_error!("invalid Max Fowards"),
        }
    }
}
