use crate::{bytes::Bytes, macros::until_newline, parser::Result};

use crate::headers::SipHeader;

use core::str;

/// The `Date` SIP header.
///
/// Reflects the time when the request or response is first sent.
pub struct Date<'a>(&'a str);

impl<'a> SipHeader<'a> for Date<'a> {
    const NAME: &'static str = "Date";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Date<'a>> {
        let date = Self::parse_as_str(bytes)?;

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
