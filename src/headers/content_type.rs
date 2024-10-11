use core::str;

use crate::{
    scanner::Scanner,
    macros::{parse_param, read_while},
    parser::{is_token, Result},
};

use super::{
    accept::{MediaType, MimeType},
    SipHeaderParser,
};
#[derive(Debug, PartialEq, Eq)]
pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeaderParser<'a> for ContentType<'a> {
    const NAME: &'static [u8] = b"Content-Type";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"c");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mtype = read_while!(scanner, is_token);
        let mtype = unsafe { str::from_utf8_unchecked(mtype) };
        scanner.next();
        let sub = read_while!(scanner, is_token);
        let sub = unsafe { str::from_utf8_unchecked(sub) };
        let param = parse_param!(scanner, |param| Some(param));

        Ok(ContentType(MediaType {
            mimetype: MimeType {
                mtype,
                subtype: sub,
            },
            param,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp\r\n";
        let mut scanner = Scanner::new(src);
    }
}
