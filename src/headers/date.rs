use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

use std::str;

pub struct Date<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Date<'a> {
    const NAME: &'static [u8] = b"Date";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let date = until_newline!(scanner);
        let date = str::from_utf8(date)?;

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