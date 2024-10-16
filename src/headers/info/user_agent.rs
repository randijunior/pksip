use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;

pub struct UserAgent<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for UserAgent<'a> {
    const NAME: &'static [u8] = b"User-Agent";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let val = until_newline!(scanner);
        let val = str::from_utf8(val)?;

        Ok(UserAgent(val))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
         let src = b"Softphone Beta1.5\r\n";
         let mut scanner = Scanner::new(src);
         let ua = UserAgent::parse(&mut scanner);
         let ua = ua.unwrap();

         assert_eq!(scanner.as_ref(), b"\r\n");
         assert_eq!(ua.0, "Softphone Beta1.5");
    }
}