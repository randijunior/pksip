use core::str;
use std::u32;

use crate::{
    bytes::Bytes,
    macros::{
        digits, parse_param, read_until_byte, sip_parse_error, space,
    },
    parser::Result,
    uri::Params,
};

use crate::headers::SipHeader;

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

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let digits = digits!(bytes);
        let digits = unsafe { str::from_utf8_unchecked(digits) };
        let digits = digits.parse::<u32>();
        space!(bytes);
        let mut comment: Option<_> = None;

        match digits {
            Ok(digits) => {
                let peeked = bytes.peek();
                if let None = peeked {
                    return sip_parse_error!("eof!");
                }
                if let Some(b'(') = peeked {
                    bytes.next();
                    let b = read_until_byte!(bytes, &b')');
                    bytes.next();
                    comment = Some(str::from_utf8(b)?);
                }
                let param = parse_param!(bytes);

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

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"18000;duration=3600\r\n";
        let mut bytes = Bytes::new(src);
        let retry_after = RetryAfter::parse(&mut bytes);
        let retry_after = retry_after.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 18000);
        assert_eq!(retry_after.param.unwrap().get("duration"), Some(&"3600"));

        let src = b"120 (I'm in a meeting)\r\n";
        let mut bytes = Bytes::new(src);
        let retry_after = RetryAfter::parse(&mut bytes);
        let retry_after = retry_after.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 120);
        assert_eq!(retry_after.comment, Some("I'm in a meeting"));
    }
}
