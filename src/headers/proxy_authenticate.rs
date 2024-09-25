use super::{authorization::Credential, SipHeaderParser};

pub struct ProxyAuthenticate<'a>(Credential<'a>);

impl<'a> SipHeaderParser<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static [u8] = b"Proxy-Authenticate";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let cred = Self::parse_auth_credential(reader)?;

        Ok(ProxyAuthenticate(cred))
    }
}
