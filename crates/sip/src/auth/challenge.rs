use crate::{
    auth::{ALGORITHM, DIGEST, DOMAIN, NONCE, OPAQUE, QOP, REALM, STALE},
    headers,
    macros::parse_comma_separated,
    parser::Result,
    scanner::Scanner,
    token::Token,
    uri::Params,
};

#[derive(Debug)]
pub enum Challenge<'a> {
    Digest(DigestChallenge<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

#[derive(Default, Debug)]
pub struct DigestChallenge<'a> {
    pub realm: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub nonce: Option<&'a str>,
    pub opaque: Option<&'a str>,
    pub stale: Option<&'a str>,
    pub algorithm: Option<&'a str>,
    pub qop: Option<&'a str>,
    pub param: Params<'a>,
}

impl<'a> Challenge<'a> {
    pub fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(scanner)?;

        if scheme == DIGEST {
            let digest = DigestChallenge::parse(scanner)?;
            return Ok(Challenge::Digest(digest));
        }

        let mut param = Params::new();
        parse_comma_separated!(scanner => {
            let (name, value) = headers::parse_header_param(scanner)?;

            param.set(name, value.unwrap_or(""));

        });

        Ok(Challenge::Other { scheme, param })
    }
}

impl<'a> DigestChallenge<'a> {
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut digest = Self::default();
        parse_comma_separated!(scanner => {
            let (name, value) = headers::parse_header_param(scanner)?;

            match name {
                REALM => digest.realm = value,
                NONCE => digest.nonce = value,
                DOMAIN => digest.domain = value,
                ALGORITHM => digest.algorithm = value,
                OPAQUE => digest.opaque = value,
                QOP => digest.qop = value,
                STALE => digest.stale = value,
                other => {
                    digest
                        .param
                        .set(other, value.unwrap_or(""));
                }
            }
        });

        Ok(digest)
    }
}
