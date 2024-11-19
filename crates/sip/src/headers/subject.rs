use std::str;

use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Subject` SIP header.
///
/// Provides a summary or indicates the nature of the call.
pub struct Subject<'a>(&'a str);

impl<'a> SipHeader<'a> for Subject<'a> {
    const NAME: &'static str = "Subject";
    const SHORT_NAME: Option<&'static str> = Some("s");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let subject = Self::parse_as_str(scanner)?;

        Ok(Subject(subject))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Need more boxes\r\n";
        let mut scanner = Scanner::new(src);
        let subject = Subject::parse(&mut scanner);
        let subject = subject.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Need more boxes");

        let src = b"Tech Support\r\n";
        let mut scanner = Scanner::new(src);
        let subject = Subject::parse(&mut scanner);
        let subject = subject.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Tech Support");
    }
}
