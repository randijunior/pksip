use std::fmt;
use std::str;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::macros::parse_header_param;
use crate::message::Parameters;
use crate::parser::Parser;
use crate::ArcStr;

/// The `Alert-Info` SIP header.
///
/// Specifies an alternative ring tone.
///
/// # Examples
///
/// ```
/// # use pksip::header::AlertInfo;
/// let info = AlertInfo::new("http://www.alert.com/sounds/moo.wav");
///
/// assert_eq!(
///     info.to_string(),
///     "Alert-Info: <http://www.alert.com/sounds/moo.wav>"
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AlertInfo {
    url: ArcStr,
    params: Option<Parameters>,
}

impl AlertInfo {
    /// Creates a new `AlertInfo` header.
    pub fn new(url: &str) -> Self {
        Self {
            url: url.into(),
            params: None,
        }
    }

    /// Creates a new `AlertInfo` header with the specified
    /// url and params.
    pub fn from_parts(url: ArcStr, params: Option<Parameters>) -> Self {
        Self { url, params }
    }

    /// Set the url for this header.
    pub fn set_url(&mut self, url: &str) {
        self.url = url.into();
    }
}

impl<'a> HeaderParser<'a> for AlertInfo {
    const NAME: &'static str = "Alert-Info";

    /*
     * Alert-Info   =  "Alert-Info" HCOLON alert-param
     * *(COMMA alert-param) alert-param  =  LAQUOT
     * absoluteURI RAQUOT *( SEMI generic-param )
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        parser.space();

        parser.next_byte()?;
        let url = parser.read_until_byte(b'>');
        parser.next_byte()?;

        let url = str::from_utf8(url)?.into();
        let params = parse_header_param!(parser);

        Ok(AlertInfo { url, params })
    }
}

impl fmt::Display for AlertInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: <{}>", AlertInfo::NAME, self.url)?;
        if let Some(params) = &self.params {
            write!(f, "{}", params)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::parser::Parser;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut scanner = Parser::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.remaining(), b"\r\n");
        assert_eq!(
            alert_info.url.as_ref(),
            "http://www.example.com/sounds/moo.wav"
        );

        let src = b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut scanner = Parser::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.remaining(), b"\r\n");
        assert_eq!(
            alert_info.url.as_ref(),
            "http://example.com/ringtones/premium.wav"
        );
        assert_eq!(
            alert_info.params.unwrap().get_named("purpose"),
            Some("ringtone")
        );
    }
}
