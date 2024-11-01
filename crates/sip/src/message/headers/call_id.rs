use crate::{
    bytes::Bytes, macros::until_newline, parser::Result, util::is_newline,
};

use crate::headers::SipHeader;

use std::str;

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

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let id = until_newline!(bytes);
        let id = str::from_utf8(id)?;

        Ok(CallId(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"bs9ki9iqbee8k5kal8mpqb\r\n";
        let mut bytes = Bytes::new(src);
        let cid = CallId::parse(&mut bytes).unwrap();

        assert_eq!(cid.id(), "bs9ki9iqbee8k5kal8mpqb");
    }
}
