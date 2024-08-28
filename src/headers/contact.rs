use crate::uri::SipUri;

use super::SipHeaderParser;

pub struct Contact<'a> {
    uri: SipUri<'a>,
    q: Option<f32>,
    expires: Option<u32>,
}

impl<'a> SipHeaderParser<'a> for Contact<'a> {
    const NAME: &'a [u8] = b"Contact";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let q = "1.0".parse::<f32>().ok();
        todo!()
    }
}
