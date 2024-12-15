use reader::{alpha, space, Reader};

use crate::{msg::SipMethod, parser::Result};

use crate::headers::SipHeader;

use std::str;

/// The `CSeq` SIP header.
///
/// Ensures order and tracking of SIP transactions within a session.
#[derive(Debug, PartialEq, Eq)]
pub struct CSeq {
    pub cseq: i32,
    pub method: SipMethod,
}

impl<'a> CSeq{
    pub fn new(cseq: i32, method: SipMethod) -> Self {
        Self { cseq, method }
    }
}

impl<'a> SipHeader<'a> for CSeq {
    const NAME: &'static str = "CSeq";

    fn parse(reader: &mut Reader<'a>) -> Result<CSeq> {
        let cseq = reader.read_num()?;

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
