use std::str;

use scanner::Scanner;

use crate::token::Token;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as perceived by the client.
pub struct Priority<'a>(&'a str);

impl<'a> SipHeader<'a> for Priority<'a> {
    const NAME: &'static str = "Priority";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let priority = Token::parse(scanner);

        Ok(Priority(priority))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"emergency\r\n";
        let mut scanner = Scanner::new(src);
        let priority = Priority::parse(&mut scanner).unwrap();

        assert_eq!(priority.0, "emergency");
    }
}
