use core::fmt;
use std::str;

use reader::Reader;

use crate::{
    headers::SipHeader,
    macros::parse_header_param,
    parser::{self, Result},
};

use crate::internal::MediaType;

/// The `Content-Type` SIP header.
///
/// Indicates the media type of the `message-body` sent to the recipient.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentType<'a>(pub MediaType<'a>);

impl<'a> SipHeader<'a> for ContentType<'a> {
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
    fn parse(reader: &mut Reader<'a>) -> Result<ContentType<'a>> {
        let mtype = parser::parse_token(reader)?;
        reader.must_read(b'/')?;
        let subtype = parser::parse_token(reader)?;
        let param = parse_header_param!(reader);
        let media_type = MediaType::new(mtype, subtype, param);

        Ok(ContentType(media_type))
    }
}

impl fmt::Display for ContentType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp\r\n";
        let mut reader = Reader::new(src);
        let c_type = ContentType::parse(&mut reader);
        let c_type = c_type.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "application");
        assert_eq!(c_type.0.mimetype.subtype, "sdp");

        let src = b"text/html; charset=ISO-8859-4\r\n";
        let mut reader = Reader::new(src);
        let c_type = ContentType::parse(&mut reader);
        let c_type = c_type.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "text");
        assert_eq!(c_type.0.mimetype.subtype, "html");
        assert_eq!(c_type.0.param.unwrap().get("charset"), Some(&"ISO-8859-4"));
    }
}
