use core::str;

use crate::token::Token;
use crate::{bytes::Bytes, macros::space, parser::Result};

use crate::headers::SipHeader;

/// Indicate `proxy-sensitive` features that must be supported by the proxy.
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ProxyRequire<'a> {
    const NAME: &'static str = "Proxy-Require";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tag = Token::parse(bytes);
        let mut tags = vec![tag];

        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let tag = Token::parse(bytes);
            tags.push(tag);
            space!(bytes);
        }

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
