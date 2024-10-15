use crate::{
    scanner::Scanner,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use crate::headers::SipHeaderParser;

use std::str;
#[derive(Debug, PartialEq, Eq)]
pub struct MaxForwards(u32);

impl<'a> SipHeaderParser<'a> for MaxForwards {
    const NAME: &'static [u8] = b"Max-Forwards";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(digits) => Ok(MaxForwards(digits)),
            Err(_) => sip_parse_error!("invalid Max Fowards"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"6\r\n";
        let mut scanner = Scanner::new(src);
        let c_length = MaxForwards::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_length, MaxForwards(6))
    }
}
