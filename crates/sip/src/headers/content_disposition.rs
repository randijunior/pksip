use core::fmt;

use reader::Reader;

use crate::parser;
use crate::{macros::parse_header_param, message::Params, parser::Result};

use crate::headers::SipHeader;

/// The `Content-Disposition` SIP header.
///
/// Describes how the `message-body` is to be interpreted by the `UAC` or `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentDisposition<'a> {
    _type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for ContentDisposition<'a> {
    const NAME: &'static str = "Content-Disposition";
    /*
     * Content-Disposition   =  "Content-Disposition" HCOLON
     *                          disp-type *( SEMI disp-param )
     * disp-type             =  "render" / "session" / "icon" / "alert"
     *                          / disp-extension-token
     * disp-param            =  handling-param / generic-param
     * handling-param        =  "handling" EQUAL
     *                          ( "optional" / "required"
     *                          / other-handling )
     * other-handling        =  token
     * disp-extension-token  =  token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let _type = parser::parse_token(reader)?;
        let params = parse_header_param!(reader);

        Ok(ContentDisposition { _type, params })
    }
}

impl fmt::Display for ContentDisposition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self._type)?;

        if let Some(param) = &self.params {
            write!(f, ";{}", param)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"session\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");

        let src = b"session;handling=optional\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");
        assert_eq!(disp.params.unwrap().get("handling"), Some(&"optional"));

        let src = b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "attachment");
        let params = disp.params.unwrap();

        assert_eq!(params.get("filename"), Some(&"smime.p7s"));
        assert_eq!(params.get("handling"), Some(&"required"));
    }
}
