use reader::Reader;
use std::fmt;

use crate::{
    internal::{ArcStr, Param},
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
pub enum Challenge {
    /// A `digest` authentication scheme.
    Digest {
        realm: Option<ArcStr>,
        domain: Option<ArcStr>,
        nonce: Option<ArcStr>,
        opaque: Option<ArcStr>,
        stale: Option<ArcStr>,
        algorithm: Option<ArcStr>,
        qop: Option<ArcStr>,
    },
    /// Other scheme not specified.
    Other { scheme: ArcStr, param: Params },
}

impl Challenge {
    ///  Use `reader` to parse a `Challenge`.
    pub fn parse(reader: &mut Reader) -> Result<Self> {
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

                match name.as_ref() {
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
            param.set(name, value.unwrap_or("".into()));

        });

        Ok(Challenge::Other {
            scheme: scheme.into(),
            param,
        })
    }
}

impl fmt::Display for Challenge {
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DigestCredential {
    pub realm: Option<ArcStr>,
    pub username: Option<ArcStr>,
    pub nonce: Option<ArcStr>,
    pub uri: Option<ArcStr>,
    pub response: Option<ArcStr>,
    pub algorithm: Option<ArcStr>,
    pub cnonce: Option<ArcStr>,
    pub opaque: Option<ArcStr>,
    pub qop: Option<ArcStr>,
    pub nc: Option<ArcStr>,
}

/// This type represent a credential containing the authentication
/// information in `Authorization` and `Proxy-Authorization` headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential {
    /// A `digest` authentication scheme.
    Digest(DigestCredential),
    /// Other scheme not specified.
    Other { scheme: ArcStr, param: Params },
}

impl fmt::Display for Credential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Credential::Digest(DigestCredential {
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
            }) => {
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

impl Credential {
    ///  Use `reader` to parse a `Credential`.
    pub fn parse(reader: &mut Reader) -> Result<Self> {
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

                match name.as_ref() {
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

            return Ok(Credential::Digest(DigestCredential {
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
            }));
        }

        comma_sep!(reader => {
            let Param {name, value} = Param::parse(reader)?;
            param.set(name, value.unwrap_or("".into()));

        });

        Ok(Credential::Other {
            scheme: scheme.into(),
            param,
        })
    }
}
