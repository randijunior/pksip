use core::str;

use crate::{
    macros::{digits, read_until_byte, read_while, sip_parse_error, space},
    parser::is_host,
};

use super::SipHeaderParser;

pub struct Warning<'a> {
    code: u32,
    host: &'a str,
    text: &'a str,
}

impl<'a> SipHeaderParser<'a> for Warning<'a> {
    const NAME: &'static [u8] = b"Warning";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let code = digits!(reader);
        let code = unsafe { str::from_utf8_unchecked(code) };
        if let Ok(code) = code.parse::<u32>() {
            space!(reader);
            let host = read_while!(reader, is_host);
            let host = unsafe { str::from_utf8_unchecked(host) };
            if let Some(b'"') = reader.read_if_eq(b'"') {
                let text = read_until_byte!(reader, b'"');
                let text = unsafe { str::from_utf8_unchecked(text) };

                Ok(Warning { code, host, text })
            } else {
                sip_parse_error!("invalid warning header!")
            }
        } else {
            sip_parse_error!("invalid warning header!")
        }
    }
}
