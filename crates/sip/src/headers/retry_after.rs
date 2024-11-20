use std::str;
use std::u32;

use scanner::space;
use scanner::until_byte;
use scanner::Scanner;

use crate::{
    macros::parse_header_param,
    parser::Result,
    uri::Params,
};

use crate::headers::SipHeader;

/// The `Retry-After` SIP header.
///
/// Indicate how long the service is expected to be
/// unavailable to the requesting client.
/// Or when the called party anticipates being available again.
pub struct RetryAfter<'a> {
    seconds: u32,
    param: Option<Params<'a>>,
    comment: Option<&'a str>,
}

impl<'a> SipHeader<'a> for RetryAfter<'a> {
    const NAME: &'static str = "Retry-After";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = scanner.read_num()?;
        let mut comment = None;

        space!(scanner);
        if let Some(&b'(') = scanner.peek() {
            scanner.next();
            let b = until_byte!(scanner, &b')');
            scanner.must_read(b')')?;
            comment = Some(str::from_utf8(b)?);
        }
        let param = parse_header_param!(scanner);

        Ok(RetryAfter {
            seconds: digits,
            param,
            comment,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"18000;duration=3600\r\n";
        let mut scanner = Scanner::new(src);
        let retry_after = RetryAfter::parse(&mut scanner);
        let retry_after = retry_after.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 18000);
        assert_eq!(retry_after.param.unwrap().get("duration"), Some(&"3600"));

        let src = b"120 (I'm in a meeting)\r\n";
        let mut scanner = Scanner::new(src);
        let retry_after = RetryAfter::parse(&mut scanner);
        let retry_after = retry_after.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 120);
        assert_eq!(retry_after.comment, Some("I'm in a meeting"));
    }
}
