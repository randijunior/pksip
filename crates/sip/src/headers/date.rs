use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

use std::{fmt, str};

/// The `Date` SIP header.
///
/// Reflects the time when the request or response is first sent.
#[derive(Debug, PartialEq, Eq)]
pub struct Date<'a>(&'a str);

impl<'a> SipHeader<'a> for Date<'a> {
    const NAME: &'static str = "Date";

    fn parse(reader: &mut Reader<'a>) -> Result<Date<'a>> {
        let date = Self::parse_as_str(reader)?;

        Ok(Date(date))
    }
}

impl fmt::Display for Date<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Sat, 13 Nov 2010 23:29:00 GMT\r\n";
        let mut reader = Reader::new(src);
        let date = Date::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(date.0, "Sat, 13 Nov 2010 23:29:00 GMT");
    }
}
