use core::str;

use crate::{
    macros::{read_while, space},
    parser::{is_token, Result},
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyRequire<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for ProxyRequire<'a> {
    const NAME: &'static [u8] = b"Proxy-Require";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let tag = read_while!(scanner, is_token);
        let tag = unsafe { str::from_utf8_unchecked(tag) };
        let mut tags = vec![tag];

        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);
            let tag = read_while!(scanner, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
            tags.push(tag);
            space!(scanner);
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
        let mut scanner = Scanner::new(src);
        let proxy_require = ProxyRequire::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(proxy_require, ProxyRequire(vec!["foo", "bar"]));
    }
}
