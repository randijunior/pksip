use core::str;

use crate::{
    bytes::Bytes,
    macros::read_while,
    parser::{is_token, Result},
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct Priority<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Priority<'a> {
    const NAME: &'static [u8] = b"Priority";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let priority = read_while!(bytes, is_token);
        let priority = unsafe { str::from_utf8_unchecked(priority) };

        Ok(Priority(priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"emergency\r\n";
        let mut bytes = Bytes::new(src);
        let priority = Priority::parse(&mut bytes).unwrap();

        assert_eq!(priority, Priority("emergency"));
    }
}
