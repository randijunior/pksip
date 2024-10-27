use core::str;

use crate::{
    bytes::Bytes,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use crate::headers::SipHeaderParser;

pub struct ContentLength(u32);

impl ContentLength {
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }
}

impl<'a> SipHeaderParser<'a> for ContentLength {
    const NAME: &'static [u8] = b"Content-Length";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"l");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let digits = digits!(bytes);
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
        let mut bytes = Bytes::new(src);
        let c_length = ContentLength::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_length.0, 349)
    }
}
