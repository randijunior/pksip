use crate::{byte_reader::ByteReader, macros::alpha, msg::SipMethod, parser::Result};

use super::SipHeaderParser;

pub struct Allow<'a>(Vec<SipMethod<'a>>);

impl<'a> SipHeaderParser<'a> for Allow<'a> {
    const NAME: &'a [u8] = b"Allow";
    
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut allow: Vec<SipMethod> = Vec::new();
        let b_method = alpha!(reader);
        let method = SipMethod::from(b_method);

        allow.push(method);

        while let Some(b',') = reader.peek() {
            reader.next();

            let b_method = alpha!(reader);
            let method = SipMethod::from(b_method);

            allow.push(method);
        }

        Ok(Allow(allow))
    }

    
}