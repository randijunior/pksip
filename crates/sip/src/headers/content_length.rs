use std::str;

use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Content-Length` SIP header.
///
/// Indicates the size of the `message-body`.
pub struct ContentLength(u32);

impl ContentLength {
    pub fn new(c_len: u32) -> Self {
        Self(c_len)
    }
}

impl<'a> SipHeader<'a> for ContentLength {
    const NAME: &'static str = "Content-Length";
    const SHORT_NAME: Option<&'static str> = Some("l");

    fn parse(scanner: &mut Scanner<'a>) -> Result<ContentLength> {
        let l = scanner.read_num()?;

        Ok(ContentLength(l))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"349\r\n";
        let mut scanner = Scanner::new(src);
        let length = ContentLength::parse(&mut scanner);
        let length = length.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(length.0, 349)
    }
}
