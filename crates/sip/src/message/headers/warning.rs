use core::str;

use crate::{
    macros::{digits, read_until_byte, read_while, sip_parse_error, space},
    uri::is_host,
};

use crate::headers::SipHeaderParser;


pub struct Warning<'a> {
    code: u32,
    host: &'a str,
    text: &'a str,
}

impl<'a> SipHeaderParser<'a> for Warning<'a> {
    const NAME: &'static [u8] = b"Warning";

    fn parse(
        bytes: &mut crate::bytes::Bytes<'a>,
    ) -> crate::parser::Result<Self> {
        let code = digits!(bytes);
        let code = unsafe { str::from_utf8_unchecked(code) };
        if let Ok(code) = code.parse::<u32>() {
            space!(bytes);
            let host = read_while!(bytes, is_host);
            let host = unsafe { str::from_utf8_unchecked(host) };
            if let Ok(Some(b'"')) = bytes.read_if(|b| b == &b'"') {
                let text = read_until_byte!(bytes, &b'"');
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
