use std::fmt;
use std::str;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::parser::Parser;
use crate::ArcStr;

/// The `Date` SIP header.
///
/// Reflects the time when the request or response is first
/// sent.
///
/// # Examples
///
/// ```
/// # use pksip::{header::Date};
/// let date = Date::new("Sat, 13 Nov 2010 23:29:00 GMT");
///
/// assert_eq!("Date: Sat, 13 Nov 2010 23:29:00 GMT", date.to_string());
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(transparent)]
pub struct Date(ArcStr);

impl Date {
    /// Create a new `Date` instance.
    pub fn new(d: &str) -> Self {
        Self(d.into())
    }
}

impl<'a> HeaderParser<'a> for Date {
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
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let date = parser.read_until_new_line()?;

        Ok(Date(date.into()))
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Date::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Sat, 13 Nov 2010 23:29:00 GMT\r\n";
        let mut scanner = Parser::new(src);
        let date = Date::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaining(), b"\r\n");
        assert_eq!(date.0.as_ref(), "Sat, 13 Nov 2010 23:29:00 GMT");
    }
}
