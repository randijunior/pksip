use crate::{
    bytes::Bytes,
    macros::{
        parse_auth_param, read_until_byte, read_while, sip_parse_error, space,
    },
    parser::{self, is_token, Result},
    uri::Params,
};

#[derive(Default)]
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
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(bytes);
            match parser::parse_token(bytes) {
                "realm" => digest.realm = parse_auth_param!(bytes),
                "username" => digest.username = parse_auth_param!(bytes),
                "nonce" => digest.nonce = parse_auth_param!(bytes),
                "uri" => digest.uri = parse_auth_param!(bytes),
                "response" => digest.response = parse_auth_param!(bytes),
                "algorithm" => digest.algorithm = parse_auth_param!(bytes),
                "cnonce" => digest.cnonce = parse_auth_param!(bytes),
                "opaque" => digest.opaque = parse_auth_param!(bytes),
                "qop" => digest.qop = parse_auth_param!(bytes),
                "nc" => digest.nc = parse_auth_param!(bytes),
                other => {
                    digest
                        .param
                        .set(other, parse_auth_param!(bytes).unwrap_or(""));
                }
            };

            if let Some(&b',') = bytes.peek() {
                bytes.next();
            } else {
                break;
            }
        }

        Ok(digest)
    }
}

pub enum Credential<'a> {
    Digest(DigestCredential<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

impl<'a> Credential<'a> {
    pub fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let scheme = match bytes.peek() {
            Some(b'"') => {
                bytes.next();
                let value = read_until_byte!(bytes, &b'"');
                bytes.next();
                value
            }
            Some(_) => {
                read_while!(bytes, is_token)
            }
            None => return sip_parse_error!("eof!"),
        };

        match scheme {
            b"Digest" => {
                Ok(Credential::Digest(DigestCredential::parse(bytes)?))
            }
            other => {
                space!(bytes);
                let other = std::str::from_utf8(other)?;
                let name = parser::parse_token(bytes);
                let val = parse_auth_param!(bytes);
                let mut params = Params::new();
                params.set(name, val.unwrap_or(""));

                while let Some(b',') = bytes.peek() {
                    space!(bytes);
                    let name = parser::parse_token(bytes);
                    let val = parse_auth_param!(bytes);
                    params.set(name, val.unwrap_or(""));
                }

                Ok(Credential::Other {
                    scheme: other,
                    param: params,
                })
            }
        }
    }
}
