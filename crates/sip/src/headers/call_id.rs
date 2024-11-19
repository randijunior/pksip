use crate::{parser::Result, scanner::Scanner};

use crate::headers::SipHeader;

use std::str;

/// The `Call-ID` SIP header.
///
/// Uniquely identifies a particular invitation or all registrations of a particular client.
pub struct CallId<'a>(&'a str);

impl<'a> From<&'a str> for CallId<'a> {
    fn from(value: &'a str) -> Self {
        Self::new(value)
    }
}

impl<'a> CallId<'a> {
    pub fn new(id: &'a str) -> Self {
        Self(id)
    }
    pub fn id(&self) -> &str {
        self.0
    }
}

impl<'a> SipHeader<'a> for CallId<'a> {
    const NAME: &'static str = "Call-ID";
    const SHORT_NAME: Option<&'static str> = Some("i");

    fn parse(scanner: &mut Scanner<'a>) -> Result<CallId<'a>> {
        let id = Self::parse_as_str(scanner)?;

        Ok(CallId(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"bs9ki9iqbee8k5kal8mpqb\r\n";
        let mut scanner = Scanner::new(src);
        let cid = CallId::parse(&mut scanner).unwrap();

        assert_eq!(cid.id(), "bs9ki9iqbee8k5kal8mpqb");
    }
}
