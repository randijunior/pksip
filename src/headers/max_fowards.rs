use crate::{byte_reader::ByteReader, macros::{digits, sip_parse_error}, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct MaxForwards(u32);

impl<'a> SipHeaderParser<'a> for MaxForwards {
    const NAME: &'a [u8] = b"Max-Forwards";
    
    fn parse(reader: &mut ByteReader<'a>) -> Result<MaxForwards> {
        let digits = digits!(reader);
        match str::from_utf8(digits)?.parse()  {
            Ok(max_f) => Ok(MaxForwards(max_f)),
            Err(_) => sip_parse_error!("invalid Max Fowards")
        }
    }
}