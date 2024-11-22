use std::str;

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Server` SIP header.
///
/// Is used by UACs to tell UASs about options
/// that the UAC expects the UAS to support in order to process the
/// request.
#[derive(Debug, PartialEq, Eq)]
pub struct Server<'a>(&'a str);

impl<'a> SipHeader<'a> for Server<'a> {
    const NAME: &'static str = "Server";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let server = Self::parse_as_str(reader)?;

        Ok(Server(server))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"HomeServer v2\r\n";
        let mut reader = Reader::new(src);
        let server = Server::parse(&mut reader);
        let server = server.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(server.0, "HomeServer v2");
    }
}
