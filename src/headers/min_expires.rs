use core::str;

use crate::{
    scanner::Scanner,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use super::SipHeaderParser;

pub struct MinExpires(u32);

impl<'a> SipHeaderParser<'a> for MinExpires {
    const NAME: &'static [u8] = b"Min-Expires";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(expires) => Ok(MinExpires(expires)),
            Err(_) => return sip_parse_error!("invalid Min-Expires!"),
        }
    }
}
