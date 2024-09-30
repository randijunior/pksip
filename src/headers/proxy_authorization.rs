use crate::{byte_reader::ByteReader, parser::Result};

use super::{authorization::Credential, SipHeaderParser};

pub struct ProxyAuthorization<'a> {
    credential: Credential<'a>
}

impl<'a> SipHeaderParser<'a> for ProxyAuthorization<'a> {
    const NAME: &'static [u8] = b"Proxy-Authorization";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let credential = Self::parse_auth_credential(reader)?;

        Ok(ProxyAuthorization { credential })
    }
}
