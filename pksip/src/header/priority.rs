use std::fmt;
use std::str;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::parser::Parser;
use crate::ArcStr;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as received by the
/// client.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Priority(ArcStr);

impl<'a> HeaderParser<'a> for Priority {
    const NAME: &'static str = "Priority";

    /*
     * Priority        =  "Priority" HCOLON priority-value
     * priority-value  =  "emergency" / "urgent" / "normal"
     *                    / "non-urgent" / other-priority
     * other-priority  =  token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let priority = parser.parse_token()?;

        Ok(Priority(priority.into()))
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Priority::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"emergency\r\n";
        let mut scanner = Parser::new(src);
        let priority = Priority::parse(&mut scanner).unwrap();

        assert_eq!(priority.0.as_ref(), "emergency");
    }
}
