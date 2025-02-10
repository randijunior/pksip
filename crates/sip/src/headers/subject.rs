use std::{fmt, str};

use reader::Reader;

use crate::internal::ArcStr;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Subject` SIP header.
///
/// Provides a summary or indicates the nature of the call.
#[derive(Debug, PartialEq, Eq)]
pub struct Subject(ArcStr);

impl SipHeader<'_> for Subject {
    const NAME: &'static str = "Subject";
    const SHORT_NAME: &'static str = "s";
    /*
     * Subject  =  ( "Subject" / "s" ) HCOLON [TEXT-UTF8-TRIM]
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let subject = Self::parse_as_str(reader)?.into();

        Ok(Subject(subject))
    }
}

impl fmt::Display for Subject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Need more boxes\r\n";
        let mut reader = Reader::new(src);
        let subject = Subject::parse(&mut reader);
        let subject = subject.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Need more boxes".into());

        let src = b"Tech Support\r\n";
        let mut reader = Reader::new(src);
        let subject = Subject::parse(&mut reader);
        let subject = subject.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Tech Support".into());
    }
}
