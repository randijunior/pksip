use core::str;


use crate::{
    bytes::Bytes, parser::Result,
};

use crate::headers::SipHeader;


/// The `Expires` SIP header.
///
/// Gives the relative time after which the message (or content) expires.
pub struct Expires(i32);

impl Expires {
    pub fn new(expires: i32) -> Self {
        Self(expires)
    }
}

impl<'a> SipHeader<'a> for Expires {
    const NAME: &'static str = "Expires";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Expires> {
        let expires = bytes.parse_num()?;

        Ok(Expires(expires))
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
