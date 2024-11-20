use scanner::Scanner;

use crate::parser::Result;

use crate::headers::SipHeader;

use std::str;

/// The `Date` SIP header.
///
/// Reflects the time when the request or response is first sent.
#[derive(Debug, PartialEq, Eq)]
pub struct Date<'a>(&'a str);

impl<'a> SipHeader<'a> for Date<'a> {
    const NAME: &'static str = "Date";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Date<'a>> {
        let date = Self::parse_as_str(scanner)?;

        Ok(Date(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Sat, 13 Nov 2010 23:29:00 GMT\r\n";
        let mut scanner = Scanner::new(src);
        let date = Date::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(date.0, "Sat, 13 Nov 2010 23:29:00 GMT");
    }
}
