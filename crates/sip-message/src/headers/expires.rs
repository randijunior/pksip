use std::{fmt, str};

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Expires` SIP header.
///
/// Gives the relative time after which the message (or content) expires.
#[derive(Debug, PartialEq, Eq)]
pub struct Expires(pub i32);

impl Expires {
    pub fn new(expires: i32) -> Self {
        Self(expires)
    }
}

impl<'a> SipHeader<'a> for Expires {
    const NAME: &'static str = "Expires";

    fn parse(reader: &mut Reader<'a>) -> Result<Expires> {
        let expires = reader.read_num()?;

        Ok(Expires(expires))
    }
}

impl fmt::Display for Expires {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"5\r\n";
        let mut reader = Reader::new(src);
        let expires = Expires::parse(&mut reader).unwrap();
        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(expires.0, 5);
    }
}
