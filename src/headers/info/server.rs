use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;
#[derive(Debug)]
pub struct Server<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Server<'a> {
    const NAME: &'static [u8] = b"Server";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let val = until_newline!(scanner);
        let val = str::from_utf8(val)?;

        Ok(Server(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
         let src = b"HomeServer v2\r\n";
         let mut scanner = Scanner::new(src);
         let server = Server::parse(&mut scanner);
         let server = server.unwrap();

         assert_eq!(scanner.as_ref(), b"\r\n");
         assert_eq!(server.0, "HomeServer v2");
    }
}
