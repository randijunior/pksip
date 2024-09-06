use crate::{byte_reader::ByteReader, macros::{alpha, digits, sip_parse_error, space}, msg::SipMethod, parser::Result};

use super::SipHeaderParser;

use std::str;
pub struct CSeq<'a> {
    cseq: i32,
    method: SipMethod<'a>
}

impl<'a> SipHeaderParser<'a> for CSeq<'a> {
    const NAME: &'a [u8] = b"CSeq";
    
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let digits = digits!(reader);
        let cseq: i32 = match str::from_utf8(digits)?.parse()  {
            Ok(cseq) => cseq,
            Err(_) => return sip_parse_error!("invalid CSeq!")
        };

        space!(reader);
        let b_method = alpha!(reader);
        let method = SipMethod::from(b_method);
        
        Ok(CSeq { cseq, method })
    }
}