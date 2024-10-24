use crate::{bytes::Bytes, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;

use std::str;

#[derive(Debug, PartialEq, Eq)]
pub struct Date<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Date<'a> {
    const NAME: &'static [u8] = b"Date";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let date = until_newline!(bytes);
        let date = str::from_utf8(date)?;

        Ok(Date(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Sat, 13 Nov 2010 23:29:00 GMT\r\n";
        let mut bytes = Bytes::new(src);
        let date = Date::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(date.0, "Sat, 13 Nov 2010 23:29:00 GMT");
    }
}
