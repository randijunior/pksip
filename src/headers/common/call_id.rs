use crate::{
    macros::until_newline, parser::Result, scanner::Scanner, util::is_newline,
};

use crate::headers::SipHeaderParser;

use std::str;
#[derive(Debug, PartialEq, Eq, Default)]
pub struct CallId<'a>(&'a str);

impl<'a> From<&'a str> for CallId<'a> {
    fn from(value: &'a str) -> Self {
        Self(value)
    }
}

impl<'a> CallId<'a> {
    pub fn id(&self) -> &str {
        self.0
    }
}

impl<'a> SipHeaderParser<'a> for CallId<'a> {
    const NAME: &'static [u8] = b"Call-ID";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"i");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let id = until_newline!(scanner);
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
        let mut scanner = Scanner::new(src);
        let cid = CallId::parse(&mut scanner).unwrap();

        assert_eq!(cid.id(), "bs9ki9iqbee8k5kal8mpqb");
    }
}
