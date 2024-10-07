use crate::{scanner::Scanner, parser::Result};

use super::{authorization::Credential, SipHeaderParser};

pub struct ProxyAuthorization<'a> {
    credential: Credential<'a>,
}

impl<'a> SipHeaderParser<'a> for ProxyAuthorization<'a> {
    const NAME: &'static [u8] = b"Proxy-Authorization";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let credential = Self::parse_auth_credential(scanner)?;

        Ok(ProxyAuthorization { credential })
    }
}
