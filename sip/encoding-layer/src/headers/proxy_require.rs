use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::macros::hdr_list;
use crate::parser::{self, Result};

use crate::headers::SipHeader;

/// The `Proxy-Require` SIP header.
///
/// Indicate `proxy-sensitive` features that must be supported by the proxy.
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ProxyRequire<'a> {
    const NAME: &'static str = "Proxy-Require";
    /*
     * Proxy-Require  =  "Proxy-Require" HCOLON option-tag
     *                   *(COMMA option-tag)
     * option-tag     =  token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = hdr_list!(reader => parser::parse_token(reader)?);

        Ok(ProxyRequire(tags))
    }
}

impl fmt::Display for ProxyRequire<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"foo, bar\r\n";
        let mut reader = Reader::new(src);
        let proxy_require = ProxyRequire::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_eq!(proxy_require.0.get(0), Some(&"foo"));
        assert_eq!(proxy_require.0.get(1), Some(&"bar"));
    }
}
