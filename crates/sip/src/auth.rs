use reader::Reader;
use std::fmt;

use crate::{
    internal::Param,
    macros::comma_sep,
    message::Params,
    parser::{self, Result},
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

/// This enum represent a challenge authentication mechanism used in
/// `Proxy-Authenticate` and `WWW-Authenticate` headers.
#[derive(Debug, PartialEq, Eq)]
pub enum Challenge<'a> {
    /// A `digest` authentication scheme.
    Digest {
        realm: Option<&'a str>,
        domain: Option<&'a str>,
        nonce: Option<&'a str>,
        opaque: Option<&'a str>,
        stale: Option<&'a str>,
        algorithm: Option<&'a str>,
        qop: Option<&'a str>,
    },
    /// Other scheme not specified.
    Other { scheme: &'a str, param: Params<'a> },
}

impl<'a> Challenge<'a> {
    ///  Use `reader` to parse a `Challenge`.
    pub fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let scheme = parser::parse_token(reader)?;
        let mut param = Params::new();

        if scheme == DIGEST {
            let mut realm = None;
            let mut nonce = None;
            let mut domain = None;
            let mut algorithm = None;
            let mut opaque = None;
            let mut qop = None;
            let mut stale = None;

            comma_sep!(reader => {
                let Param {name, value} = Param::parse(reader)?;

                match name {
                    REALM => realm = value,
                    NONCE => nonce = value,
                    DOMAIN => domain = value,
                    ALGORITHM => algorithm = value,
                    OPAQUE => opaque = value,
                    QOP => qop = value,
                    STALE => stale = value,
                    _other => {
                        // return err?
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
            });
        }

        comma_sep!(reader => {
            let Param {name, value} = Param::parse(reader)?;
            param.set(name, value.unwrap_or(""));

        });

        Ok(Challenge::Other { scheme, param })
    }
}

impl fmt::Display for Challenge<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Challenge::Digest {
                realm,
                domain,
                nonce,
                opaque,
                stale,
                algorithm,
                qop,
            } => {
                write!(f, "Digest ")?;
                if let Some(realm) = realm {
                    write!(f, "realm={realm}")?;
                }
                if let Some(domain) = domain {
                    write!(f, ", domain={domain}")?;
                }
                if let Some(nonce) = nonce {
                    write!(f, ", nonce={nonce}")?;
                }
                if let Some(opaque) = opaque {
                    write!(f, ", opaque={opaque}")?;
                }
                if let Some(stale) = stale {
                    write!(f, ", stale={stale}")?;
                }
                if let Some(algorithm) = algorithm {
                    write!(f, ", algorithm={algorithm}")?;
                }
                if let Some(qop) = qop {
                    write!(f, ", qop={qop}")?;
                }

                Ok(())
            }
            Challenge::Other { scheme, param } => todo!(),
        }
    }
}

/// This type represent a credential containing the authentication
/// information in `Authorization` and `Proxy-Authorization` headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential<'a> {
    /// A `digest` authentication scheme.
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
    },
    /// Other scheme not specified.
    Other { scheme: &'a str, param: Params<'a> },
}

impl fmt::Display for Credential<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credential::Digest {
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
            } => {
                write!(f, "Digest ")?;
                if let Some(realm) = realm {
                    write!(f, "realm={realm}")?;
                }
                if let Some(username) = username {
                    write!(f, ", username={username}")?;
                }
                if let Some(nonce) = nonce {
                    write!(f, ", nonce={nonce}")?;
                }
                if let Some(uri) = uri {
                    write!(f, ", uri={uri}")?;
                }
                if let Some(response) = response {
                    write!(f, ", response={response}")?;
                }
                if let Some(algorithm) = algorithm {
                    write!(f, ", algorithm={algorithm}")?;
                }
                if let Some(cnonce) = cnonce {
                    write!(f, ", cnonce={cnonce}")?;
                }
                if let Some(qop) = qop {
                    write!(f, ", qop={qop}")?;
                }
                if let Some(nc) = nc {
                    write!(f, ", nc={nc}")?;
                }
                if let Some(opaque) = opaque {
                    write!(f, ", opaque={opaque}")?;
                }

                Ok(())
            }
            Credential::Other { scheme, param } => todo!(),
        }
    }
}

impl<'a> Credential<'a> {
    ///  Use `reader` to parse a `Credential`.
    pub fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let scheme = parser::parse_token(reader)?;
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

            comma_sep!(reader => {
                let Param {name, value} = Param::parse(reader)?;

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
                    _other => {
                        // return err?
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
            });
        }

        comma_sep!(reader => {
            let Param {name, value} = Param::parse(reader)?;
            param.set(name, value.unwrap_or(""));

        });

        Ok(Credential::Other { scheme, param })
    }
}
