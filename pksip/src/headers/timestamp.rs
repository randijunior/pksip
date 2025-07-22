use std::{fmt, str};

use crate::error::Result;
use crate::parser::Parser;

use crate::headers::SipHeaderParse;

/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeaderParse<'a> for Timestamp<'a> {
    const NAME: &'static str = "Timestamp";
    /*
     * Timestamp  =  "Timestamp" HCOLON 1*(DIGIT)
     *                [ "." *(DIGIT) ] [ LWS delay ]
     * delay      =  *(DIGIT) [ "." *(DIGIT) ]
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let time = parser.parse_number_str();
        parser.ws();

        let delay = if parser.peek().is_some_and(|b| b.is_ascii_digit()) {
            Some(parser.parse_number_str())
        } else {
            None
        };

        Ok(Timestamp { time, delay })
    }
}

impl fmt::Display for Timestamp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Timestamp::NAME, self.time)?;

        if let Some(delay) = &self.delay {
            write!(f, "{}", delay)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"54.0 1.5\r\n";
        let mut scanner = Parser::new(src);
        let timestamp = Timestamp::parse(&mut scanner);
        let timestamp = timestamp.unwrap();

        assert_eq!(timestamp.time, "54.0");
    }
}
