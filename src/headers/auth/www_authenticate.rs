use crate::{headers::SipHeaderParser, parser::Result, scanner::Scanner};

use super::proxy_authenticate::Challenge;

pub struct WWWAuthenticate<'a> {
    challenge: Challenge<'a>,
}

impl<'a> SipHeaderParser<'a> for WWWAuthenticate<'a> {
    const NAME: &'static [u8] = b"WWW-Authenticate";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let challenge = Self::parse_auth_challenge(scanner)?;

        Ok(WWWAuthenticate { challenge })
    }
}