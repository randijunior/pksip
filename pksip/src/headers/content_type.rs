use core::fmt;
use std::str;

use crate::parser::ParseCtx;
use crate::{error::Result, headers::SipHeaderParse};

use crate::MediaType;

/// The `Content-Type` SIP header.
///
/// Indicates the media type of the `message-body` sent to
/// the recipient.
///
/// Both the long (`Content-Type`) and short (`c`) header names are supported.
///
/// # Examples
/// ```
/// # use pksip::headers::ContentType;
/// # use pksip::MediaType;
///
/// let ctype = ContentType::new(MediaType::from_static("application/sdp").unwrap());
///
/// assert_eq!(
///     "Content-Type: application/sdp",
///     ctype.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContentType<'a>(MediaType<'a>);

impl<'a> ContentType<'a> {
    /// Creates a new `Content-Type` with sdp as `MediaType`
    pub fn new_sdp() -> Self {
        Self(MediaType {
            mimetype: crate::MimeType {
                mtype: "application".into(),
                subtype: "sdp".into(),
            },
            param: None,
        })
    }
    /// Creates a new `ContentType`.
    pub fn new(m: MediaType<'a>) -> Self {
        Self(m)
    }

    /// Returns the internal `MediaType`.
    pub fn media_type(&self) -> &MediaType<'a> {
        &self.0
    }
}

impl<'a> SipHeaderParse<'a> for ContentType<'a> {
    const NAME: &'static str = "Content-Type";
    const SHORT_NAME: &'static str = "c";
    /*
     * Content-Type     =  ( "Content-Type" / "c" ) HCOLON media-type
     * media-type       =  m-type SLASH m-subtype *(SEMI m-parameter)
     * m-type           =  discrete-type / composite-type
     * discrete-type    =  "text" / "image" / "audio" / "video"
     *                     / "application" / extension-token
     * composite-type   =  "message" / "multipart" / extension-token
     * extension-token  =  ietf-token / x-token
     * ietf-token       =  token
     * x-token          =  "x-" token
     * m-subtype        =  extension-token / iana-token
     * iana-token       =  token
     * m-parameter      =  m-attribute EQUAL m-value
     * m-attribute      =  token
     * m-value          =  token / quoted-string
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let media_type = MediaType::parse(parser)?;

        Ok(ContentType(media_type))
    }
}

impl fmt::Display for ContentType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ContentType::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp\r\n";
        let mut scanner = ParseCtx::new(src);
        let c_type = ContentType::parse(&mut scanner);
        let c_type = c_type.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "application");
        assert_eq!(c_type.0.mimetype.subtype, "sdp");

        let src = b"text/html; charset=ISO-8859-4\r\n";
        let mut scanner = ParseCtx::new(src);
        let c_type = ContentType::parse(&mut scanner);
        let c_type = c_type.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "text");
        assert_eq!(c_type.0.mimetype.subtype, "html");
        assert_eq!(c_type.0.param.unwrap().get("charset").unwrap(), Some("ISO-8859-4"));
    }
}
