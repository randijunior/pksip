use std::{fmt, str};

use crate::parser::Parser;

use crate::error::Result;

use crate::headers::SipHeaderParse;

/// The `Min-Expires` SIP header.
///
/// Indicates the minimum refresh interval supported for soft-state
/// elements managed by the server.
///
/// # Examples
/// ```
/// # use pksip::headers::MinExpires;
///
/// let min_exp = MinExpires::new(90);
///
/// assert_eq!(
///     "Min-Expires: 90",
///     min_exp.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct MinExpires(u32);

impl MinExpires {
    /// Creates a new `MinExpires` header value.
    ///
    /// # Examples
    /// ```
    /// # use pksip::headers::MinExpires;
    /// let min_exp = MinExpires::new(90);
    /// assert_eq!(min_exp.as_u32(), 90);
    /// ```
    #[inline]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the `MinExpires` value as a `u32`.
    ///
    /// # Examples
    /// ```
    /// # use pksip::headers::MinExpires;
    /// let min_exp = MinExpires::new(120);
    /// assert_eq!(min_exp.as_u32(), 120);
    /// ```
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeaderParse<'a> for MinExpires {
    const NAME: &'static str = "Min-Expires";
    /*
     * Min-Expires  =  "Min-Expires" HCOLON delta-seconds
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let expires = parser.parse_u32()?;

        Ok(MinExpires(expires))
    }
}

impl fmt::Display for MinExpires {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", MinExpires::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"60";
        let mut scanner = Parser::new(src);
        let mime_version = MinExpires::parse(&mut scanner).unwrap();

        assert_eq!(mime_version.0, 60);
    }
}
