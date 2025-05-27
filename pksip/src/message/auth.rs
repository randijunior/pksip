//! SIP Auth types
//!
use std::fmt;

use crate::message::Params;

/// A Digest Challenge.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DigestChallenge<'a> {
    /// The realm of the digest authentication.
    pub realm: Option<&'a str>,

    /// The domain of the digest authentication.
    pub domain: Option<&'a str>,

    /// The nonce of the digest authentication.
    pub nonce: Option<&'a str>,

    /// The opaque value of the digest authentication.
    pub opaque: Option<&'a str>,

    /// Indicates whether the previous request was stale.
    pub stale: Option<&'a str>,

    /// The algorithm used in the digest authentication.
    pub algorithm: Option<&'a str>,

    /// The quality of protection (qop) value.
    pub qop: Option<&'a str>,
}

/// This enum represents an authentication challenge mechanism
/// used in `Proxy-Authenticate` and `WWW-Authenticate` headers.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Challenge<'a> {
    /// A `Digest` authentication scheme.
    Digest(DigestChallenge<'a>),
    /// Any other authentication scheme not specifically handled.
    Other {
        /// The name of the authentication scheme.
        scheme: &'a str,

        /// The parameters associated with the scheme.
        param: Params<'a>,
    },
}

impl fmt::Display for Challenge<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Challenge::Digest(DigestChallenge {
                realm,
                domain,
                nonce,
                opaque,
                stale,
                algorithm,
                qop,
            }) => {
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
            Challenge::Other { scheme: _, param: _ } => todo!(),
        }
    }
}

/// Represents credentials for a `Digest` authentication scheme,
/// typically found in the `Authorization` and `Proxy-Authorization` headers.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DigestCredential<'a> {
    /// The realm value that defines the protection space.
    pub realm: Option<&'a str>,

    /// The username associated with the credential.
    pub username: Option<&'a str>,

    /// The nonce value provided by the server.
    pub nonce: Option<&'a str>,

    /// The URI of the requested resource.
    pub uri: Option<&'a str>,

    /// The response hash calculated from the credential data.
    pub response: Option<&'a str>,

    /// The algorithm used to hash the credentials (e.g., "MD5").
    pub algorithm: Option<&'a str>,

    /// The client nonce value (cnonce) used to prevent replay attacks.
    pub cnonce: Option<&'a str>,

    /// The opaque value provided by the server, to be returned unchanged.
    pub opaque: Option<&'a str>,

    /// The quality of protection (qop) applied to the message.
    pub qop: Option<&'a str>,

    /// The nonce count (nc), indicating the number of requests made with the same nonce.
    pub nc: Option<&'a str>,
}

/// This type represent a credential containing the
/// authentication information in `Authorization` and
/// `Proxy-Authorization` headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credential<'a> {
    /// A `digest` authentication scheme.
    Digest(DigestCredential<'a>),
    /// Other scheme not specified.
    Other {
        /// The name of the authentication scheme.
        scheme: &'a str,

        /// The parameters associated with the scheme.
        param: Params<'a>,
    },
}

impl fmt::Display for Credential<'_> {
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
                if let Some(username) = username {
                    write!(f, "username={username}")?;
                }
                if let Some(realm) = realm {
                    write!(f, ", realm={realm}")?;
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
            Credential::Other { scheme: _, param: _ } => todo!(),
        }
    }
}
