use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;
#[derive(Debug)]
pub struct Subject<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Subject<'a> {
    const NAME: &'static [u8] = b"Subject";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"s");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let val = until_newline!(scanner);
        let val = str::from_utf8(val)?;

        Ok(Subject(val))
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
