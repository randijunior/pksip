use core::str;

use crate::{
    scanner::Scanner,
    macros::read_while,
    parser::{is_token, Result},
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Priority<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Priority<'a> {
    const NAME: &'static [u8] = b"Priority";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let priority = read_while!(scanner, is_token);
        let priority = unsafe { str::from_utf8_unchecked(priority) };

        Ok(Priority(priority))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"emergency\r\n";
        let mut scanner = Scanner::new(src);
        let priority = Priority::parse(&mut scanner).unwrap();

        assert_eq!(priority, Priority("emergency"));
    }
}