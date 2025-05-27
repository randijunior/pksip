use crate::{error::Result, headers::SipHeaderParse, macros::parse_header_param, message::Params, parser::ParseCtx};
use std::{fmt, str};

/// The `Alert-Info` SIP header.
///
/// Specifies an alternative ring tone.
///
/// # Examples
///
/// ```
/// # use pksip::headers::AlertInfo;
/// let info = AlertInfo::new("http://www.alert.com/sounds/moo.wav");
///
/// assert_eq!(
///     info.to_string(),
///     "Alert-Info: <http://www.alert.com/sounds/moo.wav>"
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> AlertInfo<'a> {
    /// Creates a new `AlertInfo` header.
    pub fn new(url: &'a str) -> Self {
        Self { url, params: None }
    }

    /// Creates a new `AlertInfo` header with the specified url and params.
    pub fn from_parts(url: &'a str, params: Option<Params<'a>>) -> Self {
        Self { url, params }
    }

    /// Set the url for this header.
    pub fn set_url(&mut self, url: &'a str) {
        self.url = url;
    }
}

impl<'a> SipHeaderParse<'a> for AlertInfo<'a> {
    const NAME: &'static str = "Alert-Info";
    /*
     * Alert-Info   =  "Alert-Info" HCOLON alert-param *(COMMA alert-param)
     * alert-param  =  LAQUOT absoluteURI RAQUOT *( SEMI generic-param )
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        parser.take_ws();

        parser.advance();
        let url = parser.read_until_byte(b'>');
        parser.advance();

        let url = str::from_utf8(url)?;
        let params = parse_header_param!(parser);

        Ok(AlertInfo { url, params })
    }
}

impl fmt::Display for AlertInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: <{}>", AlertInfo::NAME, self.url)?;
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::ParseCtx;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut scanner = ParseCtx::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");

        let src = b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut scanner = ParseCtx::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(alert_info.params.unwrap().get("purpose").unwrap(), Some("ringtone"));
    }
}
