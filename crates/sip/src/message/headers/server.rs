use core::str;

use crate::{bytes::Bytes, macros::until_newline, parser::Result};

use crate::headers::SipHeaderParser;


pub struct Server<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Server<'a> {
    const NAME: &'static [u8] = b"Server";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let val = until_newline!(bytes);
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
        let mut bytes = Bytes::new(src);
        let server = Server::parse(&mut bytes);
        let server = server.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(server.0, "HomeServer v2");
    }
}
