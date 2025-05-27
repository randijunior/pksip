use super::SipHeaderParse;
use crate::{error::Result, macros::parse_header_param, message::Params, parser::ParseCtx};
use std::{fmt, str};

const PURPOSE: &str = "purpose";

/// The `Call-Info` SIP header.
///
/// Provides aditional information aboute the caller or
/// calle.
///
/// # Examples
///
/// ```
/// # use pksip::headers::CallInfo;
/// let mut info = CallInfo::new("http://www.example.com/alice/");
///
/// assert_eq!(
///     "Call-Info: <http://www.example.com/alice/>",
///     info.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> CallInfo<'a> {
    /// Creates a new `CallInfo` header.
    pub fn new(url: &'a str) -> Self {
        Self {
            url,
            purpose: None,
            params: None,
        }
    }

    /// Creates a new `CallInfo` header with the given url, params and purpose.
    pub fn from_parts(url: &'a str, purpose: Option<&'a str>, params: Option<Params<'a>>) -> Self {
        Self { url, purpose, params }
    }
    /// Set the url for this header.
    pub fn set_url(&mut self, url: &'a str) {
        self.url = url;
    }
}

impl<'a> SipHeaderParse<'a> for CallInfo<'a> {
    const NAME: &'static str = "Call-Info";
    /*
     * Call-Info = "Call-Info" HCOLON info * (COMMA info)
     * info = LAQUOT absoluteURI RAQUOT * (SEMI info-param)
     * info-param = ("purpose" EQUAL ("icon" | "info" | "card" | token)) |
     *		        generic-param
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        parser.advance();
        let url = parser.read_until_byte(b'>');
        // must be an '>'
        parser.advance();
        let url = str::from_utf8(url)?;
        let params = parse_header_param!(parser, PURPOSE = purpose);

        Ok(CallInfo { url, params, purpose })
    }
}

impl fmt::Display for CallInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: <{}>", CallInfo::NAME, self.url)?;
        if let Some(purpose) = &self.purpose {
            write!(f, ";{}", purpose)?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> \
        ;purpose=icon\r\n";
        let mut scanner = ParseCtx::new(src);
        let info = CallInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon".into()));

        let src = b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut scanner = ParseCtx::new(src);
        let info = CallInfo::parse(&mut scanner).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info".into()));
    }
}
