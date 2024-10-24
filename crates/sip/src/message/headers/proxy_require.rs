use core::str;

use crate::{
    bytes::Bytes,
    macros::space,
    parser::{self, Result},
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for ProxyRequire<'a> {
    const NAME: &'static [u8] = b"Proxy-Require";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tag = parser::parse_token(bytes);
        let mut tags = vec![tag];

        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let tag = parser::parse_token(bytes);
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
        assert_eq!(proxy_require, ProxyRequire(vec!["foo", "bar"]));
    }
}
