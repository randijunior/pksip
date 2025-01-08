use super::{Header, ParseHeaderError, SipHeader};
use crate::parser::Result;
use core::fmt;
use reader::Reader;
use std::str;

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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallId<'a>(pub &'a str);

impl<'a> From<&'a str> for CallId<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> CallId<'a> {
    /// Creates a new `CallId` instance with the given identifier.
    pub fn new(id: &'a str) -> Self {
        Self(id)
    }

    /// Returns the internal `CallId` identifier.
    pub fn id(&self) -> &str {
        self.0
    }
}

impl<'a> SipHeader<'a> for CallId<'a> {
    const NAME: &'static str = "Call-ID";
    const SHORT_NAME: &'static str = "i";
    /*
     * Call-ID  =  ( "Call-ID" / "i" ) HCOLON callid
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let id = Self::parse_as_str(reader)?;

        Ok(CallId(id))
    }
}

impl<'a> TryFrom<&'a [u8]> for CallId<'a> {
    type Error = ParseHeaderError;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_call_id()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl fmt::Display for CallId<'_> {
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
