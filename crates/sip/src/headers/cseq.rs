use scanner::{alpha, space, Scanner};

use crate::{
    message::SipMethod,
    parser::Result,
};

use crate::headers::SipHeader;

use std::str;

/// The `CSeq` SIP header.
///
/// Ensures order and tracking of SIP transactions within a session.
#[derive(Debug, PartialEq, Eq)]
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

    fn parse(scanner: &mut Scanner<'a>) -> Result<CSeq<'a>> {
        let cseq = scanner.read_num()?;

        space!(scanner);
        let b_method = alpha!(scanner);
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
        let mut scanner = Scanner::new(src);
        let c_length = CSeq::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_length.method, SipMethod::Invite);
        assert_eq!(c_length.cseq, 4711);
    }
}
