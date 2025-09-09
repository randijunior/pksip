use std::{fmt, str};

use crate::error::Result;
use crate::header::HeaderParser;
use crate::parser::Parser;

/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
#[derive(Debug, PartialEq, Clone)]
pub struct Timestamp {
    time: f32,
    delay: Option<f32>,
}

impl<'a> HeaderParser<'a> for Timestamp {
    const NAME: &'static str = "Timestamp";

    /*
     * Timestamp  =  "Timestamp" HCOLON 1*(DIGIT)
     *                [ "." *(DIGIT) ] [ LWS delay ]
     * delay      =  *(DIGIT) [ "." *(DIGIT) ]
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let time = parser.read_f32()?;
        parser.skip_ws();

        let delay = if parser.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            Some(parser.read_f32()?)
        } else {
            None
        };

        Ok(Timestamp { time, delay })
    }
}

impl fmt::Display for Timestamp {
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

        assert_eq!(timestamp.time, 54.0);
    }
}
