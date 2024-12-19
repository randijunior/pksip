use std::{fmt, str};

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Subject` SIP header.
///
/// Provides a summary or indicates the nature of the call.
#[derive(Debug, PartialEq, Eq)]
pub struct Subject<'a>(&'a str);

impl<'a> SipHeader<'a> for Subject<'a> {
    const NAME: &'static str = "Subject";
    const SHORT_NAME: &'static str = "s";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let subject = Self::parse_as_str(reader)?;

        Ok(Subject(subject))
    }
}

impl fmt::Display for Subject<'_> {
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
        assert_eq!(subject.0, "Need more boxes");

        let src = b"Tech Support\r\n";
        let mut reader = Reader::new(src);
        let subject = Subject::parse(&mut reader);
        let subject = subject.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Tech Support");
    }
}
