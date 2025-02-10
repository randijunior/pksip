use std::{fmt, str};

use reader::Reader;

use crate::internal::ArcStr;
use crate::parser::{self, Result};

use crate::headers::SipHeader;

/// The `Priority` SIP header.
///
/// Indicates the urgency of the request as received by the client.
#[derive(Debug, PartialEq, Eq)]
pub struct Priority(ArcStr);

impl SipHeader<'_> for Priority {
    const NAME: &'static str = "Priority";
    /*
     * Priority        =  "Priority" HCOLON priority-value
     * priority-value  =  "emergency" / "urgent" / "normal"
     *                    / "non-urgent" / other-priority
     * other-priority  =  token
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let priority = parser::parse_token(reader)?;

        Ok(Priority(priority.into()))
    }
}

impl fmt::Display for Priority {
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

        assert_eq!(priority.0, "emergency".into());
    }
}
