use crate::parser::ParseCtx;

use crate::error::Result;

use crate::headers::SipHeaderParse;

use std::{fmt, str};

/// The `Max-Forwards` SIP header.
///
/// Limits the number of proxies or gateways that can forward
/// the request.
///
/// # Examples
/// ```
/// # use pksip::headers::MaxForwards;
///
/// let max = MaxForwards::new(70);
///
/// assert_eq!(
///     "Max-Forwards: 70",
///     max.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct MaxForwards(u32);

impl MaxForwards {
    /// Creates a new `MaxForwards` header with the given number of forwards.
    pub const fn new(fowards: u32) -> Self {
        Self(fowards)
    }
    /// Returns the internal `MaxForwards` value.
    pub fn max_fowards(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeaderParse<'a> for MaxForwards {
    const NAME: &'static str = "Max-Forwards";
    /*
     * Max-Forwards  =  "Max-Forwards" HCOLON 1*DIGIT
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<MaxForwards> {
        let fowards = parser.parse_u32()?;

        Ok(MaxForwards(fowards))
    }
}

impl fmt::Display for MaxForwards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", MaxForwards::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"6\r\n";
        let mut scanner = ParseCtx::new(src);
        let c_length = MaxForwards::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(c_length.0, 6)
    }
}
