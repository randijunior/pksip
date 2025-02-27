use reader::{alpha, space, Reader};

use crate::{message::SipMethod, parser::Result};

use crate::headers::SipHeader;

use core::fmt;
use std::str;

/// The `CSeq` SIP header.
///
/// Ensures order and tracking of SIP transactions within a session.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CSeq {
    pub cseq: u32,
    pub method: SipMethod,
}

impl fmt::Display for CSeq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.cseq, self.method)
    }
}

impl CSeq {
    pub fn new(cseq: u32, method: SipMethod) -> Self {
        Self { cseq, method }
    }
}

impl<'a> SipHeader<'a> for CSeq {
    const NAME: &'static str = "CSeq";
    /*
     * CSeq  =  "CSeq" HCOLON 1*DIGIT LWS Method
     */
    fn parse(reader: &mut Reader<'a>) -> Result<CSeq> {
        let cseq = reader.read_u32()?;

        space!(reader);
        let b_method = alpha!(reader);
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
        let mut reader = Reader::new(src);
        let c_length = CSeq::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(c_length.method, SipMethod::Invite);
        assert_eq!(c_length.cseq, 4711);
    }
}
