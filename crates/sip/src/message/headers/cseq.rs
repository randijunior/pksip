use crate::{
    bytes::Bytes,
    macros::{alpha, space},
    message::SipMethod,
    parser::Result,
};

use crate::headers::SipHeader;

use core::str;

/// The `CSeq` SIP header.
///
/// Ensures order and tracking of SIP transactions within a session.
pub struct CSeq<'a> {
    cseq: i32,
    method: SipMethod<'a>,
}

impl<'a> CSeq<'a> {
    pub fn new(cseq: i32, method: SipMethod<'a>) -> Self {
        Self { cseq, method }
    }
}

impl<'a> SipHeader<'a> for CSeq<'a> {
    const NAME: &'static str = "CSeq";

    fn parse(bytes: &mut Bytes<'a>) -> Result<CSeq<'a>> {
        let cseq = bytes.read_num()?;

        space!(bytes);
        let b_method = alpha!(bytes);
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
        let mut bytes = Bytes::new(src);
        let c_length = CSeq::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_length.method, SipMethod::Invite);
        assert_eq!(c_length.cseq, 4711);
    }
}
