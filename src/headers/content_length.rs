use core::str;

use crate::{
    scanner::Scanner,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

pub struct ContentLength(u32);

impl<'a> SipHeaderParser<'a> for ContentLength {
    const NAME: &'static [u8] = b"Content-Length";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"l");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        let digits = unsafe { str::from_utf8_unchecked(digits) };
        if let Ok(cl) = digits.parse() {
            Ok(ContentLength(cl))
        } else {
            sip_parse_error!("invalid content length")
        }
    }
}
