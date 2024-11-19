use crate::{
    auth::{
        ALGORITHM, CNONCE, DIGEST, NC, NONCE, OPAQUE, QOP, REALM, RESPONSE,
        URI, USERNAME,
    },
    headers,
    macros::parse_comma_separated,
    parser::Result,
    scanner::Scanner,
    token::Token,
    uri::Params,
};

/// This type represent a credential containing the authentication
/// information in `Authorization` and `Proxy-Authorization` headers.
#[derive(Debug, Clone)]
pub enum Credential<'a> {
    Digest(DigestCredential<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

impl<'a> Credential<'a> {
    ///  Use `scanner` to parse a `Credential`.
    pub fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(scanner)?;

        if scheme == DIGEST {
            let digest = DigestCredential::parse(scanner)?;
            return Ok(Credential::Digest(digest));
        }

        let mut param = Params::new();
        parse_comma_separated!(scanner => {
            let (name, value) = headers::parse_header_param(scanner)?;

            param.set(name, value.unwrap_or(""));

        });

        Ok(Credential::Other { scheme, param })
    }
}
/// This type represent a digest credential used in `Authorization`
/// and `Proxy-Authorization` headers for authentication.
#[derive(Default, Debug, Clone)]
pub struct DigestCredential<'a> {
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

impl<'a> DigestCredential<'a> {
    ///  Use `scanner` to parse a `DigestCredential`.
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut digest = Self::default();

        parse_comma_separated!(scanner => {
            let (name, value) = headers::parse_header_param(scanner)?;

            match name {
                REALM => digest.realm = value,
                USERNAME => digest.username = value,
                NONCE => digest.nonce = value,
                URI => digest.uri = value,
                RESPONSE => digest.response = value,
                ALGORITHM => digest.algorithm = value,
                CNONCE => digest.cnonce = value,
                OPAQUE => digest.opaque = value,
                QOP => digest.qop = value,
                NC => digest.nc = value,
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
