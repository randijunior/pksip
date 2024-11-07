use core::str;

use crate::token::Token;
use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as perceived by the client.
pub struct Priority<'a>(&'a str);

impl<'a> SipHeader<'a> for Priority<'a> {
    const NAME: &'static str = "Priority";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let priority = Token::parse(bytes);

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

        assert_eq!(priority.0, "emergency");
    }
}
