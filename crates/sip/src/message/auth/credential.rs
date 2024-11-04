use crate::{
    bytes::Bytes, headers, macros::parse_comma_separated,
    message::auth::DIGEST_SCHEME, parser::Result, token::Token, uri::Params,
};

pub enum Credential<'a> {
    Digest(Digest<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

impl<'a> Credential<'a> {
    pub fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(bytes)?;

        if scheme == DIGEST_SCHEME {
            let digest = Digest::parse(bytes)?;
            return Ok(Credential::Digest(digest));
        }

        let mut param = Params::new();
        parse_comma_separated!(bytes => {
            let (name, value) = headers::parse_param(bytes)?;

            param.set(name, value.unwrap_or(""));

        });

        Ok(Credential::Other { scheme, param })
    }
}

#[derive(Default, Debug)]
pub struct Digest<'a> {
    pub realm: Option<&'a str>,
    pub username: Option<&'a str>,
    pub nonce: Option<&'a str>,
    pub uri: Option<&'a str>,
    pub response: Option<&'a str>,
    pub algorithm: Option<&'a str>,
    pub cnonce: Option<&'a str>,
    pub opaque: Option<&'a str>,
    pub qop: Option<&'a str>,
    pub nc: Option<&'a str>,
    pub param: Params<'a>,
}

impl<'a> Digest<'a> {
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut digest = Self::default();

        parse_comma_separated!(bytes => {
            let (name, value) = headers::parse_param(bytes)?;

            match name {
                "realm" => digest.realm = value,
                "username" => digest.username = value,
                "nonce" => digest.nonce = value,
                "uri" => digest.uri = value,
                "response" => digest.response = value,
                "algorithm" => digest.algorithm = value,
                "cnonce" => digest.cnonce = value,
                "opaque" => digest.opaque = value,
                "qop" => digest.qop = value,
                "nc" => digest.nc = value,
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
