use std::{fmt, str};

use crate::error::Result;
use crate::parser::ParseCtx;

use crate::headers::SipHeaderParse;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as received by the
/// client.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Priority<'a>(&'a str);

impl<'a> SipHeaderParse<'a> for Priority<'a> {
    const NAME: &'static str = "Priority";
    /*
     * Priority        =  "Priority" HCOLON priority-value
     * priority-value  =  "emergency" / "urgent" / "normal"
     *                    / "non-urgent" / other-priority
     * other-priority  =  token
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let priority = parser.parse_token()?;

        Ok(Priority(priority))
    }
}

impl fmt::Display for Priority<'_> {
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
        let mut scanner = ParseCtx::new(src);
        let priority = Priority::parse(&mut scanner).unwrap();

        assert_eq!(priority.0, "emergency");
    }
}
