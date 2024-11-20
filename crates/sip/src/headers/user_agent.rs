use std::str;

use scanner::Scanner;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `User-Agent` SIP header.
///
/// Contains information about the `UAC` originating the request.
#[derive(Debug, PartialEq, Eq)]
pub struct UserAgent<'a>(&'a str);

impl<'a> SipHeader<'a> for UserAgent<'a> {
    const NAME: &'static str = "User-Agent";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let agent = Self::parse_as_str(scanner)?;

        Ok(UserAgent(agent))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Softphone Beta1.5\r\n";
        let mut scanner = Scanner::new(src);
        let ua = UserAgent::parse(&mut scanner);
        let ua = ua.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(ua.0, "Softphone Beta1.5");
    }
}
