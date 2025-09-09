use std::sync::Arc;
use std::{fmt, str};

use crate::error::Result;
use crate::header::HeaderParser;
use crate::parser::Parser;

/// The `Server` SIP header.
///
/// Is used by UACs to tell UASs about options
/// that the UAC expects the UAS to support in order to
/// process the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Server(Arc<str>);

impl Server {
    /// Creates a new `Server` header with the given value.
    pub fn new(s: &str) -> Self {
        Self(s.into())
    }
}

impl<'a> HeaderParser<'a> for Server {
    const NAME: &'static str = "Server";

    /*
     * Server           =  "Server" HCOLON server-val *(LWS
     * server-val) server-val       =  product / comment
     * product          =  token [SLASH product-version]
     * product-version  =  token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let server = parser.read_until_new_line_as_str()?;

        Ok(Server(server.into()))
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Server::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"HomeServer v2\r\n";
        let mut scanner = Parser::new(src);
        let server = Server::parse(&mut scanner);
        let server = server.unwrap();

        assert_eq!(scanner.remaining(), b"\r\n");
        assert_eq!(server.0.as_ref(), "HomeServer v2");
    }
}
