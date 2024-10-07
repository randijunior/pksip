use core::str;

use crate::{
    scanner::Scanner,
    macros::{read_while, space},
    parser::{is_token, Result},
};

use super::SipHeaderParser;

pub struct ContentEncoding<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for ContentEncoding<'a> {
    const NAME: &'static [u8] = b"Content-Encoding";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"e");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut codings: Vec<&'a str> = Vec::new();
        let coding = read_while!(scanner, is_token);
        let content_coding = unsafe { str::from_utf8_unchecked(coding) };
        codings.push(content_coding);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            let coding = read_while!(scanner, is_token);
            let content_coding = unsafe { str::from_utf8_unchecked(coding) };
            codings.push(content_coding);
            space!(scanner);
        }

        Ok(ContentEncoding(codings))
    }
}
