use std::str;

use crate::{
    bytes::Bytes,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use crate::headers::SipHeaderParser;


pub struct Expires(i32);

impl Expires {
    pub fn new(expires: i32) -> Self {
        Self(expires)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut bytes = Bytes::new(bytes);

        Self::parse(&mut bytes)
    }
}

impl<'a> SipHeaderParser<'a> for Expires {
    const NAME: &'static [u8] = b"Expires";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let digits = digits!(bytes);
        match str::from_utf8(digits)?.parse() {
            Ok(expires) => Ok(Expires(expires)),
            Err(_) => return sip_parse_error!("invalid Expires!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"5\r\n";
        let mut bytes = Bytes::new(src);
        let expires = Expires::parse(&mut bytes).unwrap();
        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(expires.0, 5);
    }
}
