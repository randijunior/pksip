use core::str;

use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// The `Server` SIP header.
///
/// Is used by UACs to tell UASs about options
/// that the UAC expects the UAS to support in order to process the
/// request.
pub struct Server<'a>(&'a str);

impl<'a> SipHeader<'a> for Server<'a> {
    const NAME: &'static str = "Server";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let server = Self::parse_as_str(bytes)?;

        Ok(Server(server))
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
