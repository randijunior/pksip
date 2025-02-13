use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::internal::ArcStr;
use crate::macros::hdr_list;
use crate::parser::{self, Result};

use crate::headers::SipHeader;

/// The `Proxy-Require` SIP header.
///
/// Indicate `proxy-sensitive` features that must be supported by the proxy.
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyRequire(Vec<ArcStr>);

impl SipHeader<'_> for ProxyRequire {
    const NAME: &'static str = "Proxy-Require";
    /*
     * Proxy-Require  =  "Proxy-Require" HCOLON option-tag
     *                   *(COMMA option-tag)
     * option-tag     =  token
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let tags = hdr_list!(reader => parser::parse_token(reader)?.into());

        Ok(ProxyRequire(tags))
    }
}

impl fmt::Display for ProxyRequire {
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

        assert_eq!(proxy_require.0.get(0), Some(&"foo".into()));
        assert_eq!(proxy_require.0.get(1), Some(&"bar".into()));
    }
}
