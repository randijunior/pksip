use std::str;

use reader::Reader;

use crate::parser::Result;
use crate::token::Token;

use crate::headers::SipHeader;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as perceived by the client.
#[derive(Debug, PartialEq, Eq)]
pub struct Priority<'a>(&'a str);

impl<'a> SipHeader<'a> for Priority<'a> {
    const NAME: &'static str = "Priority";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let priority = Token::parse(reader)?;

        Ok(Priority(priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"emergency\r\n";
        let mut reader = Reader::new(src);
        let priority = Priority::parse(&mut reader).unwrap();

        assert_eq!(priority.0, "emergency");
    }
}
