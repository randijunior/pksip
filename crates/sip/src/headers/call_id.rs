use super::{Header, ParseHeaderError, SipHeader};
use crate::{
    internal::ArcStr,
    parser::{Result, SipParserError},
};
use core::fmt;
use reader::Reader;
use std::str::{self, FromStr};

/// The `Call-ID` SIP header.
///
/// Uniquely identifies a particular invitation or all registrations of a particular client.
///
/// # Examples
///
/// ```
/// # use sip::headers::CallId;
/// let cid = CallId::new("bs9ki9iqbee8k5kal8mpqb");
///
/// assert_eq!("Call-ID: bs9ki9iqbee8k5kal8mpqb".as_bytes().try_into(), Ok(cid));
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CallId(pub ArcStr);

impl From<&str> for CallId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl CallId {
    /// Creates a new `CallId` instance with the given identifier.
    pub fn new(id: &str) -> Self {
        Self(id.into())
    }

    /// Returns the internal `CallId` identifier.
    pub fn id(&self) -> &str {
        &self.0
    }
}

impl SipHeader<'_> for CallId {
    const NAME: &'static str = "Call-ID";
    const SHORT_NAME: &'static str = "i";
    /*
     * Call-ID  =  ( "Call-ID" / "i" ) HCOLON callid
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let id = Self::parse_as_str(reader)?;

        Ok(CallId(id.into()))
    }
}

impl FromStr for CallId {
    type Err = SipParserError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(&mut Reader::new(s.as_bytes()))
    }
}

impl TryFrom<&[u8]> for CallId {
    type Error = ParseHeaderError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Header::from_bytes(value)?
            .into_call_id()
            .map_err(|_| ParseHeaderError(Self::NAME))
    }
}

impl fmt::Display for CallId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"bs9ki9iqbee8k5kal8mpqb\r\n";
        let mut reader = Reader::new(src);
        let cid = CallId::parse(&mut reader).unwrap();

        assert_eq!(cid.id(), "bs9ki9iqbee8k5kal8mpqb");
    }
}
