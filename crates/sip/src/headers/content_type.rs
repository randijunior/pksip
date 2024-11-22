use std::str;

use reader::Reader;

use crate::{
    headers::SipHeader, macros::parse_header_param, parser::Result,
    token::Token,
};

use super::MediaType;

/// The `Content-Type` SIP header.
///
/// Indicates the media type of the `message-body` sent to the recipient.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeader<'a> for ContentType<'a> {
    const NAME: &'static str = "Content-Type";
    const SHORT_NAME: Option<&'static str> = Some("c");

    fn parse(reader: &mut Reader<'a>) -> Result<ContentType<'a>> {
        let mtype = Token::parse(reader);
        reader.must_read(b'/')?;
        let subtype = Token::parse(reader);
        let param = parse_header_param!(reader);
        let media_type = MediaType::new(mtype, subtype, param);

        Ok(ContentType(media_type))
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
