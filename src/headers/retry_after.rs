use core::str;
use std::u32;

use crate::{
    byte_reader::ByteReader,
    macros::{digits, parse_param, read_until_byte, sip_parse_error},
    parser::Result,
    uri::Params,
};

use super::SipHeaderParser;

pub struct RetryAfter<'a> {
    seconds: u32,
    param: Option<Params<'a>>,
    comment: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for RetryAfter<'a> {
    const NAME: &'static [u8] = b"Retry-After";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        let digits = unsafe { str::from_utf8_unchecked(digits) };
        let digits = digits.parse::<u32>();
        let mut comment: Option<_> = None;

        match digits {
            Ok(digits) => {
                let peeked = reader.peek();
                if let None = peeked {
                    return sip_parse_error!("eof!");
                }
                if let Some(b'(') = peeked {
                    reader.next();
                    let bytes = read_until_byte!(reader, b')');
                    reader.next();
                    comment = Some(str::from_utf8(bytes)?);
                }
                let param = parse_param!(reader, |param| Some(param));

                Ok(RetryAfter {
                    seconds: digits,
                    param,
                    comment,
                })
            }
            Err(_) => todo!(),
        }
    }
}
