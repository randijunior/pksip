use std::str;

use crate::macros::{digits, sip_parse_error};

use super::SipHeaderParser;

pub struct Expires(i32);

impl<'a> SipHeaderParser<'a> for Expires {
    const NAME: &'a [u8] = b"Expires";
    
    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let digits = digits!(reader);
        match str::from_utf8(digits)?.parse()  {
            Ok(expires) => Ok(Expires(expires)),
            Err(_) => return sip_parse_error!("invalid Expires!")
        }
    }
}