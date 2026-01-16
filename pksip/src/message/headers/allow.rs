use std::fmt;

use itertools::Itertools;

use crate::{
    error::Result,
    macros::comma_separated_header_value,
    message::SipMethod,
    parser::{HeaderParser, Parser},
};

/// The `Allow` SIP header.
///
/// Indicates what methods is supported by the `UserAgent`.
///
/// # Examples
///
/// ```
/// # use pksip::header::Allow;
/// # use pksip::message::SipMethod;
/// let mut allow = Allow::new();
///
/// allow.push(SipMethod::Invite);
/// allow.push(SipMethod::Register);
///
/// assert_eq!("Allow: INVITE, REGISTER", allow.to_string());
/// ```
#[derive(Debug, PartialEq, Eq, Default, Clone)]
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

impl HeaderParser for Allow {
    const NAME: &'static str = "Allow";

    fn parse(parser: &mut Parser) -> Result<Self> {
        let allow = comma_separated_header_value!(parser => {
            let b_method = parser.alphabetic();

            SipMethod::from(b_method)
        });

        Ok(Allow(allow))
    }
}

impl fmt::Display for Allow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Allow::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"INVITE, ACK, OPTIONS, CANCEL, BYE\r\n";
        let mut scanner = Parser::new(src);
        let allow = Allow::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaining(), b"\r\n");

        assert_eq!(allow.get(0), Some(&SipMethod::Invite));
        assert_eq!(allow.get(1), Some(&SipMethod::Ack));
        assert_eq!(allow.get(2), Some(&SipMethod::Options));
        assert_eq!(allow.get(3), Some(&SipMethod::Cancel));
        assert_eq!(allow.get(4), Some(&SipMethod::Bye));
        assert_eq!(allow.get(5), None);
    }
}
