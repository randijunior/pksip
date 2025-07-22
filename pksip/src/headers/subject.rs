use std::{fmt, str};

use crate::error::Result;
use crate::parser::Parser;

use crate::headers::SipHeaderParse;

use super::Header;

/// The `Subject` SIP header.
///
/// Provides a summary or indicates the nature of the call.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Subject<'a>(&'a str);

impl<'a> SipHeaderParse<'a> for Subject<'a> {
    const NAME: &'static str = "Subject";
    const SHORT_NAME: &'static str = "s";
    /*
     * Subject  =  ( "Subject" / "s" ) HCOLON [TEXT-UTF8-TRIM]
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let subject = parser.parse_header_str()?;

        Ok(Subject(subject))
    }
}

impl fmt::Display for Subject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Subject::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Need more boxes\r\n";
        let mut scanner = Parser::new(src);
        let subject = Subject::parse(&mut scanner);
        let subject = subject.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(subject.0, "Need more boxes");

        let src = b"Tech Support\r\n";
        let mut scanner = Parser::new(src);
        let subject = Subject::parse(&mut scanner);
        let subject = subject.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(subject.0, "Tech Support");
    }
}
