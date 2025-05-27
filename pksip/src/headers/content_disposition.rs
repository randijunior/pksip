use super::SipHeaderParse;
use crate::{error::Result, macros::parse_header_param, message::Params, parser::ParseCtx};
use core::fmt;

/// The `Content-Disposition` SIP header.
///
/// Describes how the `message-body` is to be interpreted by
/// the `UAC` or `UAS`.
///
/// # Examples
///
/// ```
/// # use pksip::headers::ContentDisposition;
/// let c_disp = ContentDisposition::new("session");
///
/// assert_eq!(
///     "Content-Disposition: session",
///     c_disp.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContentDisposition<'a> {
    _type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> ContentDisposition<'a> {
    /// Creates a new `ContentDisposition` instance.
    pub fn new(_type: &'a str) -> Self {
        Self { _type, params: None }
    }
}

impl<'a> SipHeaderParse<'a> for ContentDisposition<'a> {
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
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let _type = parser.parse_token()?;
        let params = parse_header_param!(parser);

        Ok(ContentDisposition { _type, params })
    }
}

impl fmt::Display for ContentDisposition<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ContentDisposition::NAME, self._type)?;

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
        let mut scanner = ParseCtx::new(src);
        let disp = ContentDisposition::parse(&mut scanner);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");

        let src = b"session;handling=optional\r\n";
        let mut scanner = ParseCtx::new(src);
        let disp = ContentDisposition::parse(&mut scanner);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");
        assert_eq!(disp.params.unwrap().get("handling").unwrap(), Some("optional"));

        let src = b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut scanner = ParseCtx::new(src);
        let disp = ContentDisposition::parse(&mut scanner);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "attachment");
        let params = disp.params.unwrap();

        assert_eq!(params.get("filename").unwrap(), Some("smime.p7s"));
        assert_eq!(params.get("handling").unwrap(), Some("required"));
    }
}
