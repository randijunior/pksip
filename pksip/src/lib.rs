//! # pksip
//!
//! A rust library that implements the SIP protocol.

pub mod endpoint;
pub mod headers;
pub mod message;
pub mod parser;
pub mod service;
pub mod transaction;
pub mod transport;

pub(crate) mod error;
pub(crate) mod macros;

pub use endpoint::Endpoint;
use error::Error;
pub use error::Result;
use parser::ParseCtx;
pub use service::SipService;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

use std::{
    fmt,
    net::SocketAddr,
    str::{self, FromStr},
};

use crate::{error::SipParserError, message::Params};

/// Represents a quality value (q-value) used in SIP
/// headers.
///
/// The `Q` struct provides a method to parse a string
/// representation of a q-value into a `Q` instance. The
/// q-value is typically used to indicate the preference
/// of certain SIP headers.
///
/// # Example
///
/// ```
/// use pksip::Q;
///
/// let q_value = "0.5".parse();
/// assert_eq!(q_value, Ok(Q(0, 5)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Q(pub u8, pub u8);

impl Q {
    pub fn new(a: u8, b: u8) -> Self {
        Self(a, b)
    }
}
impl From<u8> for Q {
    fn from(value: u8) -> Self {
        Self(value, 0)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseQError;

impl From<ParseQError> for Error {
    fn from(value: ParseQError) -> Self {
        Self::ParseError(SipParserError {
            message: format!("{:?}", value),
        })
    }
}

impl FromStr for Q {
    type Err = ParseQError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.rsplit_once('.') {
            Some((a, b)) => {
                let a = a.parse().map_err(|_| ParseQError)?;
                let b = b.parse().map_err(|_| ParseQError)?;
                Ok(Q(a, b))
            }
            None => match s.parse() {
                Ok(n) => Ok(Q(n, 0)),
                Err(_) => Err(ParseQError),
            },
        }
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ";q={}.{}", self.0, self.1)
    }
}

/// This type reprents an MIME type that indicates an
/// content format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}

/// The `media-type` that appears in `Accept` and
/// `Content-Type` SIP headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}

impl fmt::Display for MediaType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MediaType { mimetype, param } = self;
        write!(f, "{}/{}", mimetype.mtype, mimetype.subtype)?;
        if let Some(param) = &param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

impl<'a> MediaType<'a> {
    /// Constructs a `MediaType` from a type and a subtype.
    pub fn new(mtype: &'a str, subtype: &'a str) -> Self {
        Self {
            mimetype: MimeType { mtype, subtype },
            param: None,
        }
    }

    pub fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let mtype = parser.parse_token()?;
        parser.advance();
        let subtype = parser.parse_token()?;
        let param = crate::macros::parse_header_param!(parser);

        Ok(Self::from_parts(mtype, subtype, param))
    }

    pub fn from_static(s: &'static str) -> Result<Self> {
        Self::parse(&mut ParseCtx::new(s.as_bytes()))
    }

    /// Constructs a `MediaType` with an optional
    /// parameters.
    pub fn from_parts(mtype: &'a str, subtype: &'a str, param: Option<Params<'a>>) -> Self {
        Self {
            mimetype: MimeType { mtype, subtype },
            param,
        }
    }
}

pub(crate) fn get_local_name(addr: &SocketAddr) -> String {
    let ip = local_ip_address::local_ip().unwrap_or(addr.ip());
    let local_name = format!("{}:{}", ip, addr.port());

    local_name
}
