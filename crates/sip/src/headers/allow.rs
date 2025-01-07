use std::fmt;

use itertools::Itertools;
use reader::{alpha, Reader};

use crate::{
    headers::SipHeader, macros::hdr_list, message::SipMethod, parser::Result,
};

use super::{Header, ParseHeaderError};

/// The `Allow` SIP header.
///
/// Indicates what methods is supported by the `UA`.
///
/// # Examples
///
/// ```
/// # use sip::headers::Allow;
/// # use sip::message::SipMethod;
/// let mut allow = Allow::new();
/// allow.push(SipMethod::Invite);
/// allow.push(SipMethod::Register);
///
/// assert_eq!("INVITE, REGISTER".as_bytes().try_into(), Ok(allow));
/// ```
#[derive(Debug, PartialEq, Eq, Default)]
pub struct Allow(Vec<SipMethod>);

impl Allow {
    /// Creates a empty `Allow` header.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Appends an new `SipMethod`.
    pub fn push(&mut self, method: SipMethod) {
        self.0.push(method);
    }

    /// Gets the `SipMethod` at the specified index.
    pub fn get(&self, index: usize) -> Option<&SipMethod> {
        self.0.get(index)
    }

    /// Returns the number of `SipMethods` in the header.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeader<'a> for Allow {
    const NAME: &'static str = "Allow";
    /*
     * Allow  =  "Allow" HCOLON [Method *(COMMA Method)]
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let allow = hdr_list!(reader => {
            let b_method = alpha!(reader);

            SipMethod::from(b_method)
        });

        Ok(Allow(allow))
    }
}

impl TryFrom<&[u8]> for Allow {
    type Error = ParseHeaderError;

    fn try_from(value: &[u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_allow()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl fmt::Display for Allow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"INVITE, ACK, OPTIONS, CANCEL, BYE\r\n";
        let mut reader = Reader::new(src);
        let allow = Allow::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_eq!(allow.get(0), Some(&SipMethod::Invite));
        assert_eq!(allow.get(1), Some(&SipMethod::Ack));
        assert_eq!(allow.get(2), Some(&SipMethod::Options));
        assert_eq!(allow.get(3), Some(&SipMethod::Cancel));
        assert_eq!(allow.get(4), Some(&SipMethod::Bye));
        assert_eq!(allow.get(5), None);
    }
}
