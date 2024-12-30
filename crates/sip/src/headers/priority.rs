use std::{fmt, str};

use reader::Reader;

use crate::parser::{self, Result};

use crate::headers::SipHeader;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as received by the client.
#[derive(Debug, PartialEq, Eq)]
pub struct Priority<'a>(&'a str);

impl<'a> SipHeader<'a> for Priority<'a> {
    const NAME: &'static str = "Priority";
    /*
     * Priority        =  "Priority" HCOLON priority-value
     * priority-value  =  "emergency" / "urgent" / "normal"
     *                    / "non-urgent" / other-priority
     * other-priority  =  token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let priority = parser::parse_token(reader)?;

        Ok(Priority(priority))
    }
}

impl fmt::Display for Priority<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
