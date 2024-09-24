use super::{authorization::Credential, SipHeaderParser};

pub struct ProxyAuthorization<'a>(Credential<'a>);

impl<'a> SipHeaderParser<'a> for ProxyAuthorization<'a> {
    const NAME: &'a [u8] = b"Proxy-Authorization";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let cred = Self::parse_auth_credential(reader)?;

        Ok(ProxyAuthorization(cred))
    }
}
