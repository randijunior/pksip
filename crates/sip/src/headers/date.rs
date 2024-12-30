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
    /*
     * Date          =  "Date" HCOLON SIP-date
     * SIP-date      =  rfc1123-date
     * rfc1123-date  =  wkday "," SP date1 SP time SP "GMT"
     * date1         =  2DIGIT SP month SP 4DIGIT
     *                  ; day month year (e.g., 02 Jun 1982)
     * time          =  2DIGIT ":" 2DIGIT ":" 2DIGIT
     *                  ; 00:00:00 - 23:59:59
     * wkday         =  "Mon" / "Tue" / "Wed"
     *                  / "Thu" / "Fri" / "Sat" / "Sun"
     * month         =  "Jan" / "Feb" / "Mar" / "Apr" ...
     */
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
