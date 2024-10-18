use std::str;

use crate::{
    macros::{digits, sip_parse_error},
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct Expires(i32);

impl Expires {
    pub fn new(expires: i32) -> Self {
        Self(expires)
    }
}

impl<'a> SipHeaderParser<'a> for Expires {
    const NAME: &'static [u8] = b"Expires";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
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
        let mut scanner = Scanner::new(src);
        let expires = Expires::parse(&mut scanner).unwrap();
        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(expires.0, 5);
    }
}
