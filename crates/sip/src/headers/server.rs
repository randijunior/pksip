use std::str;

use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

/// The `Server` SIP header.
///
/// Is used by UACs to tell UASs about options
/// that the UAC expects the UAS to support in order to process the
/// request.
pub struct Server<'a>(&'a str);

impl<'a> SipHeader<'a> for Server<'a> {
    const NAME: &'static str = "Server";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let server = Self::parse_as_str(scanner)?;

        Ok(Server(server))
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
