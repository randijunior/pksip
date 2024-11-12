use std::str;

use crate::macros::parse_header_list;
use crate::token::Token;
use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// The `Proxy-Require` SIP header.
///
/// Indicate `proxy-sensitive` features that must be supported by the proxy.
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ProxyRequire<'a> {
    const NAME: &'static str = "Proxy-Require";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tags = parse_header_list!(bytes => Token::parse(bytes));

        Ok(ProxyRequire(tags))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"foo, bar\r\n";
        let mut bytes = Bytes::new(src);
        let proxy_require = ProxyRequire::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");

        assert_eq!(proxy_require.0.get(0), Some(&"foo"));
        assert_eq!(proxy_require.0.get(1), Some(&"bar"));
    }
}
