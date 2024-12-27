use std::{fmt, str};

use reader::Reader;

use crate::parser::Result;

use crate::headers::SipHeader;

/// The `User-Agent` SIP header.
///
/// Contains information about the `UAC` originating the request.
#[derive(Debug, PartialEq, Eq)]
pub struct UserAgent<'a>(&'a str);

impl<'a> SipHeader<'a> for UserAgent<'a> {
    const NAME: &'static str = "User-Agent";
    /*
     * User-Agent  =  "User-Agent" HCOLON server-val *(LWS server-val)
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let agent = Self::parse_as_str(reader)?;

        Ok(UserAgent(agent))
    }
}

impl fmt::Display for UserAgent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Softphone Beta1.5\r\n";
        let mut reader = Reader::new(src);
        let ua = UserAgent::parse(&mut reader);
        let ua = ua.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(ua.0, "Softphone Beta1.5");
    }
}
