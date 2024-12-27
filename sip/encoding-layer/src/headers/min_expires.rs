use std::{fmt, str};

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Min-Expires` SIP header.
///
/// The minimum refresh interval supported for soft-state elements managed by that server.
#[derive(Debug, PartialEq, Eq)]
pub struct MinExpires(u32);

impl<'a> SipHeader<'a> for MinExpires {
    const NAME: &'static str = "Min-Expires";
    /*
     * Min-Expires  =  "Min-Expires" HCOLON delta-seconds
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let expires = reader.read_num()?;

        Ok(MinExpires(expires))
    }
}

impl fmt::Display for MinExpires {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"60";
        let mut reader = Reader::new(src);
        let mime_version = MinExpires::parse(&mut reader).unwrap();

        assert_eq!(mime_version.0, 60);
    }
}
