use crate::parser::Parser;
use crate::{error::Result, message::SipMethod};

use crate::headers::SipHeaderParse;

use core::fmt;
use std::str;

/// The `CSeq` SIP header.
///
/// Ensures order and tracking of SIP transactions within a
/// session.
///
/// # Examples
///
/// ```
/// # use pksip::{headers::CSeq, message::SipMethod};
/// let cseq = CSeq::new(1, SipMethod::Options);
///
/// assert_eq!(
///     "CSeq: 1 OPTIONS",
///     cseq.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CSeq {
    /// The CSeq number.
    pub cseq: u32,
    /// The CSeq method.
    pub method: SipMethod,
}

impl fmt::Display for CSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", CSeq::NAME, self.cseq, self.method)
    }
}

impl CSeq {
    /// Creates a new `CSeq` instance.
    pub fn new(cseq: u32, method: SipMethod) -> Self {
        Self { cseq, method }
    }

    /// Returns the cseq number.
    pub fn cseq(&self) -> u32 {
        self.cseq
    }

    /// Returns the SIP method associated with the cseq.
    pub fn method(&self) -> &SipMethod {
        &self.method
    }
}

impl<'a> SipHeaderParse<'a> for CSeq {
    const NAME: &'static str = "CSeq";
    /*
     * CSeq  =  "CSeq" HCOLON 1*DIGIT LWS SipMethod
     */
    fn parse(parser: &mut Parser<'a>) -> Result<CSeq> {
        let cseq = parser.parse_u32()?;

        parser.ws();
        let b_method = parser.alphabetic();
        let method = SipMethod::from(b_method);

        Ok(CSeq { cseq, method })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"4711 INVITE\r\n";
        let mut scanner = Parser::new(src);
        let c_length = CSeq::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(c_length.method, SipMethod::Invite);
        assert_eq!(c_length.cseq, 4711);
    }
}
