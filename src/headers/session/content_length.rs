use core::str;

use crate::{
    macros::{digits, sip_parse_error},
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct ContentLength(u32);

impl ContentLength {
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"349\r\n";
        let mut scanner = Scanner::new(src);
        let c_length = ContentLength::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_length, ContentLength(349))
    }
}
