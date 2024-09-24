use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

pub struct MinExpires(u32);

impl<'a> SipHeaderParser<'a> for MinExpires {
    const NAME: &'a [u8] = b"Min-Expires";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(expires) => Ok(MinExpires(expires)),
            Err(_) => return sip_parse_error!("invalid Min-Expires!"),
        }
    }
}
