use std::str;

use crate::{bytes::Bytes, parser::Result};

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

    fn parse(bytes: &mut Bytes<'a>) -> Result<ContentLength> {
        let l = bytes.parse_num()?;

        Ok(ContentLength(l))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"349\r\n";
        let mut bytes = Bytes::new(src);
        let length = ContentLength::parse(&mut bytes);
        let length = length.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(length.0, 349)
    }
}
