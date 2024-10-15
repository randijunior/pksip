use core::str;
use std::u32;

use crate::{
    macros::{digits, parse_param, read_until_byte, sip_parse_error, space},
    parser::Result,
    scanner::Scanner,
    uri::Params,
};

use super::SipHeaderParser;
#[derive(Debug)]
pub struct RetryAfter<'a> {
    seconds: u32,
    param: Option<Params<'a>>,
    comment: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for RetryAfter<'a> {
    const NAME: &'static [u8] = b"Retry-After";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        let digits = unsafe { str::from_utf8_unchecked(digits) };
        let digits = digits.parse::<u32>();
        space!(scanner);
        let mut comment: Option<_> = None;

        match digits {
            Ok(digits) => {
                let peeked = scanner.peek();
                if let None = peeked {
                    return sip_parse_error!("eof!");
                }
                if let Some(b'(') = peeked {
                    scanner.next();
                    let bytes = read_until_byte!(scanner, b')');
                    scanner.next();
                    comment = Some(str::from_utf8(bytes)?);
                }
                let param = parse_param!(scanner, |param| Some(param));

                Ok(RetryAfter {
                    seconds: digits,
                    param,
                    comment,
                })
            }
            Err(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"18000;duration=3600\r\n";
        let mut scanner = Scanner::new(src);
        let retry_after = RetryAfter::parse(&mut scanner);
        let retry_after = retry_after.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 18000);
        assert_eq!(
            retry_after.param,
            Some(Params::from(HashMap::from([("duration", Some("3600"))])))
        );

        let src = b"120 (I'm in a meeting)\r\n";
        let mut scanner = Scanner::new(src);
        let retry_after = RetryAfter::parse(&mut scanner);
        let retry_after = retry_after.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 120);
        assert_eq!(retry_after.comment, Some("I'm in a meeting"));
    }
}
