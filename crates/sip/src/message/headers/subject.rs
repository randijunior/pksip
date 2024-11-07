use core::str;

use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// Provides a summary or indicates the nature of the call.
pub struct Subject<'a>(&'a str);

impl<'a> SipHeader<'a> for Subject<'a> {
    const NAME: &'static str = "Subject";
    const SHORT_NAME: Option<&'static str> = Some("s");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let subject = Self::parse_as_str(bytes)?;

        Ok(Subject(subject))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Need more boxes\r\n";
        let mut bytes = Bytes::new(src);
        let subject = Subject::parse(&mut bytes);
        let subject = subject.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Need more boxes");

        let src = b"Tech Support\r\n";
        let mut bytes = Bytes::new(src);
        let subject = Subject::parse(&mut bytes);
        let subject = subject.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(subject.0, "Tech Support");
    }
}
