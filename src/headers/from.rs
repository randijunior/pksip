use crate::uri::SipUri;

use super::SipHeaderParser;

pub struct From<'a> {
    pub(crate) tag: &'a str,
    pub(crate) uri: SipUri<'a>,
}

impl<'a> SipHeaderParser<'a> for From<'a> {
    const NAME: &'a [u8] = b"From";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"f");
    
    fn parse(
        reader: &mut crate::byte_reader::ByteReader<'a>,
    ) -> crate::parser::Result<From<'a>> {

        todo!()
    }
}
