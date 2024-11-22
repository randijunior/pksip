use std::str;

use reader::Reader;

use crate::macros::parse_header_list;
use crate::token::Token;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Proxy-Require` SIP header.
///
/// Indicate `proxy-sensitive` features that must be supported by the proxy.
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ProxyRequire<'a> {
    const NAME: &'static str = "Proxy-Require";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = parse_header_list!(reader => Token::parse(reader));

        Ok(ProxyRequire(tags))
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
