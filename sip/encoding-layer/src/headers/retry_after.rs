use std::fmt;
use std::str;
use std::u32;

use reader::space;
use reader::until;
use reader::Reader;

use crate::{macros::parse_header_param, message::Params, parser::Result};

use crate::headers::SipHeader;

/// The `Retry-After` SIP header.
///
/// Indicate how long the service is expected to be
/// unavailable to the requesting client.
/// Or when the called party anticipates being available again.
#[derive(Debug, PartialEq, Eq)]
pub struct RetryAfter<'a> {
    seconds: u32,
    param: Option<Params<'a>>,
    comment: Option<&'a str>,
}

impl<'a> SipHeader<'a> for RetryAfter<'a> {
    const NAME: &'static str = "Retry-After";
    /*
     * Retry-After  =  "Retry-After" HCOLON delta-seconds
     *                 [ comment ] *( SEMI retry-param )
     * retry-param  =  ("duration" EQUAL delta-seconds)
     *                 / generic-param
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let digits = reader.read_num()?;
        let mut comment = None;

        space!(reader);
        if let Some(&b'(') = reader.peek() {
            reader.next();
            let b = until!(reader, &b')');
            reader.must_read(b')')?;
            comment = Some(str::from_utf8(b)?);
        }
        let param = parse_header_param!(reader);

        Ok(RetryAfter {
            seconds: digits,
            param,
            comment,
        })
    }
}

impl fmt::Display for RetryAfter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.seconds)?;

        if let Some(param) = &self.param {
            write!(f, ";{}", param)?;
        }
        if let Some(comment) = self.comment {
            write!(f, "{}", comment)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"18000;duration=3600\r\n";
        let mut reader = Reader::new(src);
        let retry_after = RetryAfter::parse(&mut reader);
        let retry_after = retry_after.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 18000);
        assert_eq!(retry_after.param.unwrap().get("duration"), Some(&"3600"));

        let src = b"120 (I'm in a meeting)\r\n";
        let mut reader = Reader::new(src);
        let retry_after = RetryAfter::parse(&mut reader);
        let retry_after = retry_after.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(retry_after.seconds, 120);
        assert_eq!(retry_after.comment, Some("I'm in a meeting"));
    }
}
