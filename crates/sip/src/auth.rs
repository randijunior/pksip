use reader::Reader;

use crate::{
    headers, macros::parse_comma_separated, parser::Result,
    token::Token, uri::Params,
};

pub(crate) const CNONCE: &str = "cnonce";
pub(crate) const QOP: &str = "qop";
pub(crate) const NC: &str = "nc";
pub(crate) const NEXTNONCE: &str = "nextnonce";
pub(crate) const RSPAUTH: &str = "rspauth";

const DIGEST: &str = "Digest";
const REALM: &str = "realm";
const USERNAME: &str = "username";
const NONCE: &str = "nonce";
const URI: &str = "uri";
const RESPONSE: &str = "response";
const ALGORITHM: &str = "algorithm";
const OPAQUE: &str = "opaque";
const DOMAIN: &str = "domain";
const STALE: &str = "stale";

/// This type represent a challenge authentication mechanism used in
/// `Proxy-Authenticate` and `WWW-Authenticate` headers.
#[derive(Debug, PartialEq, Eq)]
pub enum Challenge<'a> {
    Digest {
        realm: Option<&'a str>,
        domain: Option<&'a str>,
        nonce: Option<&'a str>,
        opaque: Option<&'a str>,
        stale: Option<&'a str>,
        algorithm: Option<&'a str>,
        qop: Option<&'a str>,
        param: Params<'a>,
    },
    Other {
        scheme: &'a str,
        param: Params<'a>,
    },
}

impl<'a> Challenge<'a> {
    ///  Use `reader` to parse a `Challenge`.
    pub fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(reader)?;
        let mut param = Params::new();

        if scheme == DIGEST {
            let mut realm = None;
            let mut nonce = None;
            let mut domain = None;
            let mut algorithm = None;
            let mut opaque = None;
            let mut qop = None;
            let mut stale = None;

            parse_comma_separated!(reader => {
                let (name, value) = headers::parse_header_param(reader)?;

                match name {
                    REALM => realm = value,
                    NONCE => nonce = value,
                    DOMAIN => domain = value,
                    ALGORITHM => algorithm = value,
                    OPAQUE => opaque = value,
                    QOP => qop = value,
                    STALE => stale = value,
                    other => {
                        param
                            .set(other, value.unwrap_or(""));
                    }
                }
            });

            return Ok(Challenge::Digest {
                realm,
                domain,
                nonce,
                opaque,
                stale,
                algorithm,
                qop,
                param,
            });
        }

        parse_comma_separated!(reader => {
            let (name, value) = headers::parse_header_param(reader)?;

            param.set(name, value.unwrap_or(""));

        });

        Ok(Challenge::Other { scheme, param })
    }
}

/// This type represent a credential containing the authentication
/// information in `Authorization` and `Proxy-Authorization` headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential<'a> {
    Digest {
        realm: Option<&'a str>,
        username: Option<&'a str>,
        nonce: Option<&'a str>,
        uri: Option<&'a str>,
        response: Option<&'a str>,
        algorithm: Option<&'a str>,
        cnonce: Option<&'a str>,
        opaque: Option<&'a str>,
        qop: Option<&'a str>,
        nc: Option<&'a str>,
        param: Params<'a>,
    },
    Other {
        scheme: &'a str,
        param: Params<'a>,
    },
}

impl<'a> Credential<'a> {
    ///  Use `reader` to parse a `Credential`.
    pub fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let scheme = Token::parse_quoted(reader)?;
        let mut param = Params::new();

        if scheme == DIGEST {
            let mut realm = None;
            let mut username = None;
            let mut nonce = None;
            let mut uri = None;
            let mut response = None;
            let mut algorithm = None;
            let mut cnonce = None;
            let mut opaque = None;
            let mut qop = None;
            let mut nc = None;

            parse_comma_separated!(reader => {
                let (name, value) = headers::parse_header_param(reader)?;

                match name {
                    REALM => realm = value,
                    USERNAME => username = value,
                    NONCE => nonce = value,
                    URI => uri = value,
                    RESPONSE => response = value,
                    ALGORITHM => algorithm = value,
                    CNONCE => cnonce = value,
                    OPAQUE => opaque = value,
                    QOP => qop = value,
                    NC => nc = value,
                    other => {
                        param
                            .set(other, value.unwrap_or(""));
                    }
                }
            });

            return Ok(Credential::Digest {
                realm,
                username,
                nonce,
                uri,
                response,
                algorithm,
                cnonce,
                opaque,
                qop,
                nc,
                param,
            });
        }

        parse_comma_separated!(reader => {
            let (name, value) = headers::parse_header_param(reader)?;
            param.set(name, value.unwrap_or(""));

        });

        Ok(Credential::Other { scheme, param })
    }
}
