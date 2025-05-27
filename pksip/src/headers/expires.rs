use std::{fmt, str};

use crate::error::Result;
use crate::headers::SipHeaderParse;
use crate::parser::ParseCtx;

/// The `Expires` SIP header.
///
/// Gives the relative time after which the message (or
/// content) expires.
///
/// # Examples
/// ```
/// # use pksip::{headers::Expires};
/// let expires = Expires::new(3600);
///
/// assert_eq!(
///     "Expires: 3600",
///     expires.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct Expires(u32);

impl Expires {
    /// Creates a new `Expires` header with the given expiration time.
    pub fn new(expires: u32) -> Self {
        Self(expires)
    }

    /// Returns the `Expires` value as a `u32`.
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeaderParse<'a> for Expires {
    const NAME: &'static str = "Expires";
    /*
     * Expires     =  "Expires" HCOLON delta-seconds
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Expires> {
        let expires = parser.parse_u32()?;

        Ok(Expires(expires))
    }
}

impl fmt::Display for Expires {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Expires::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"5\r\n";
        let mut scanner = ParseCtx::new(src);
        let expires = Expires::parse(&mut scanner).unwrap();
        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(expires.0, 5);
    }
}
