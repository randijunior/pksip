use crate::{
    bytes::Bytes,
    headers,
    macros::parse_comma_separated,
    message::auth::{
        ALGORITHM, DIGEST, DOMAIN, NONCE, OPAQUE, QOP, REALM, STALE,
    },
    parser::Result,
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
    pub fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(bytes)?;

        if scheme == DIGEST {
            let digest = DigestChallenge::parse(bytes)?;
            return Ok(Challenge::Digest(digest));
        }

        let mut param = Params::new();
        parse_comma_separated!(bytes => {
            let (name, value) = headers::parse_param(bytes)?;

            param.set(name, value.unwrap_or(""));

        });

        Ok(Challenge::Other { scheme, param })
    }
}

impl<'a> DigestChallenge<'a> {
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut digest = Self::default();
        parse_comma_separated!(bytes => {
            let (name, value) = headers::parse_param(bytes)?;

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
