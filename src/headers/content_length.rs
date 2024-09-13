use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

pub struct ContentLength(u32);

impl<'a> SipHeaderParser<'a> for ContentLength {
    const NAME: &'a [u8] = b"Content-Length";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"l");

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        let digits = unsafe { str::from_utf8_unchecked(digits) };
        if let Ok(cl) = digits.parse() {
            Ok(ContentLength(cl))
        } else {
            sip_parse_error!("invalid content length")
        }
    }
}
