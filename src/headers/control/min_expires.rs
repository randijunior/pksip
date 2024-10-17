use core::str;

use crate::{
    macros::{digits, sip_parse_error},
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct MinExpires(u32);

impl<'a> SipHeaderParser<'a> for MinExpires {
    const NAME: &'static [u8] = b"Min-Expires";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(expires) => Ok(MinExpires(expires)),
            Err(_) => return sip_parse_error!("invalid Min-Expires!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"60";
        let mut scanner = Scanner::new(src);
        let mime_version = MinExpires::parse(&mut scanner).unwrap();

        assert_eq!(mime_version, MinExpires(60));
    }
}
