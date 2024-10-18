use crate::{
    macros::{alpha, digits, sip_parse_error, space},
    msg::SipMethod,
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

use std::str;

#[derive(Debug, PartialEq, Eq)]
pub struct CSeq<'a> {
    cseq: i32,
    method: SipMethod<'a>,
}

impl<'a> CSeq<'a> {
    pub fn new(cseq: i32, method: SipMethod<'a>) -> Self {
        Self {cseq, method }
    }
}

impl<'a> SipHeaderParser<'a> for CSeq<'a> {
    const NAME: &'static [u8] = b"CSeq";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        let cseq: i32 = match str::from_utf8(digits)?.parse() {
            Ok(cseq) => cseq,
            Err(_) => return sip_parse_error!("invalid CSeq!"),
        };

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
