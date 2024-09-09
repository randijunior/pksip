use crate::uri::Params;

use super::SipHeaderParser;

pub struct AuthenticationInfo<'a> {
    auth_infos: Params<'a>
}

impl<'a> SipHeaderParser<'a> for AuthenticationInfo<'a> {
    const NAME: &'a [u8] = b"Authentication-Info";
    
    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        todo!()
    }
}

