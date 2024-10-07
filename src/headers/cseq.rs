use crate::{
    scanner::Scanner,
    macros::{alpha, digits, sip_parse_error, space},
    msg::SipMethod,
    parser::Result,
};

use super::SipHeaderParser;

use std::str;
pub struct CSeq<'a> {
    cseq: i32,
    method: SipMethod<'a>,
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
