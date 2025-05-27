use std::{fmt, str};

use itertools::Itertools;

use crate::error::Result;
use crate::macros::hdr_list;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Proxy-Require` SIP header.
///
/// Indicate `proxy-sensitive` features that must be
/// supported by the proxy.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeaderParse<'a> for ProxyRequire<'a> {
    const NAME: &'static str = "Proxy-Require";
    /*
     * Proxy-Require  =  "Proxy-Require" HCOLON option-tag
     *                   *(COMMA option-tag)
     * option-tag     =  token
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let tags = hdr_list!(parser => parser.parse_token()?);

        Ok(ProxyRequire(tags))
    }
}

impl fmt::Display for ProxyRequire<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ProxyRequire::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"foo, bar\r\n";
        let mut scanner = ParseCtx::new(src);
        let proxy_require = ProxyRequire::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");

        assert_eq!(proxy_require.0.get(0), Some(&"foo".into()));
        assert_eq!(proxy_require.0.get(1), Some(&"bar".into()));
    }
}
