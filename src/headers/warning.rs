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

    fn parse(scanner: &mut crate::scanner::Scanner<'a>) -> crate::parser::Result<Self> {
        let code = digits!(scanner);
        let code = unsafe { str::from_utf8_unchecked(code) };
        if let Ok(code) = code.parse::<u32>() {
            space!(scanner);
            let host = read_while!(scanner, is_host);
            let host = unsafe { str::from_utf8_unchecked(host) };
            if let Ok(Some(b'"')) = scanner.read_if_eq(b'"') {
                let text = read_until_byte!(scanner, b'"');
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
