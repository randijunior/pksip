use std::{fmt, str};

use crate::error::Result;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

use super::Header;

/// The `Server` SIP header.
///
/// Is used by UACs to tell UASs about options
/// that the UAC expects the UAS to support in order to
/// process the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Server<'a>(&'a str);

impl<'a> Server<'a> {
    /// Creates a new `Server` header with the given value.
    pub fn new(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'a> SipHeaderParse<'a> for Server<'a> {
    const NAME: &'static str = "Server";
    /*
     * Server           =  "Server" HCOLON server-val *(LWS server-val)
     * server-val       =  product / comment
     * product          =  token [SLASH product-version]
     * product-version  =  token
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let server = parser.parse_header_value_as_str()?;

        Ok(Server(server))
    }
}

impl fmt::Display for Server<'_> {
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
        let mut scanner = ParseCtx::new(src);
        let server = Server::parse(&mut scanner);
        let server = server.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(server.0, "HomeServer v2");
    }
}
