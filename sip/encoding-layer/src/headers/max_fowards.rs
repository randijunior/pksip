use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

use std::{fmt, str};

/// The `Max-Forwards` SIP header.
///
/// Limit the number of proxies or gateways that can forward the request.
#[derive(Debug, PartialEq, Eq)]
pub struct MaxForwards(u32);

impl MaxForwards {
    pub fn new(fowards: u32) -> Self {
        Self(fowards)
    }
    pub fn max_fowards(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeader<'a> for MaxForwards {
    const NAME: &'static str = "Max-Forwards";
    /*
     * Max-Forwards  =  "Max-Forwards" HCOLON 1*DIGIT
     */
    fn parse(reader: &mut Reader<'a>) -> Result<MaxForwards> {
        let fowards = reader.read_num()?;

        Ok(MaxForwards(fowards))
    }
}

impl fmt::Display for MaxForwards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"6\r\n";
        let mut reader = Reader::new(src);
        let c_length = MaxForwards::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(c_length.0, 6)
    }
}
