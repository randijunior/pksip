use std::str;

use crate::{bytes::Bytes, parser::Result};

use crate::headers::SipHeader;

/// The `User-Agent` SIP header.
///
/// Contains information about the `UAC` originating the request.
pub struct UserAgent<'a>(&'a str);

impl<'a> SipHeader<'a> for UserAgent<'a> {
    const NAME: &'static str = "User-Agent";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let agent = Self::parse_as_str(bytes)?;

        Ok(UserAgent(agent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Softphone Beta1.5\r\n";
        let mut bytes = Bytes::new(src);
        let ua = UserAgent::parse(&mut bytes);
        let ua = ua.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(ua.0, "Softphone Beta1.5");
    }
}
