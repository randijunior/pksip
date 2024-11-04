use crate::{
    bytes::Bytes,
    macros::{
        parse_auth_param, read_until_byte, read_while, sip_parse_error, space,
    },
    parser::Result,
    token::{is_token, Token},
    uri::Params,
};

pub enum Challenge<'a> {
    Digest(DigestChallenge<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

#[derive(Default)]
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
            b"Digest" => Ok(Challenge::Digest(DigestChallenge::parse(bytes)?)),
            other => {
                space!(bytes);
                let other = std::str::from_utf8(other)?;
                let name = Token::parse(bytes);
                let val = parse_auth_param!(bytes);
                let mut params = Params::new();
                params.set(name, val.unwrap_or(""));

                while let Some(b',') = bytes.peek() {
                    space!(bytes);

                    let name = Token::parse(bytes);
                    let val = parse_auth_param!(bytes);
                    params.set(name, val.unwrap_or(""));
                }

                Ok(Challenge::Other {
                    scheme: other,
                    param: params,
                })
            }
        }
    }
}

impl<'a> DigestChallenge<'a> {
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(bytes);
            match Token::parse(bytes) {
                "realm" => digest.realm = parse_auth_param!(bytes),
                "nonce" => digest.nonce = parse_auth_param!(bytes),
                "domain" => digest.domain = parse_auth_param!(bytes),
                "algorithm" => digest.algorithm = parse_auth_param!(bytes),
                "opaque" => digest.opaque = parse_auth_param!(bytes),
                "qop" => digest.qop = parse_auth_param!(bytes),
                "stale" => digest.stale = parse_auth_param!(bytes),
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
