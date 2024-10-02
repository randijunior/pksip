use crate::{byte_reader::ByteReader, parser::Result};

use super::{proxy_authenticate::Challenge, SipHeaderParser};

pub struct WWWAuthenticate<'a> {
    challenge: Challenge<'a>,
}

impl<'a> SipHeaderParser<'a> for WWWAuthenticate<'a> {
    const NAME: &'static [u8] = b"WWW-Authenticate";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let challenge = Self::parse_auth_challenge(reader)?;

        Ok(WWWAuthenticate { challenge })
    }
}
