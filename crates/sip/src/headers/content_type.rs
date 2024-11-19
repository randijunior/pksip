use std::str;

use crate::{
    headers::SipHeader, macros::parse_header_param, parser::Result,
    scanner::Scanner, token::Token,
};

use super::MediaType;

/// The `Content-Type` SIP header.
///
/// Indicates the media type of the `message-body` sent to the recipient.
pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeader<'a> for ContentType<'a> {
    const NAME: &'static str = "Content-Type";
    const SHORT_NAME: Option<&'static str> = Some("c");

    fn parse(scanner: &mut Scanner<'a>) -> Result<ContentType<'a>> {
        let mtype = Token::parse(scanner);
        scanner.must_read(b'/')?;
        let subtype = Token::parse(scanner);
        let param = parse_header_param!(scanner);
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
        let mut scanner = Scanner::new(src);
        let c_type = ContentType::parse(&mut scanner);
        let c_type = c_type.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "application");
        assert_eq!(c_type.0.mimetype.subtype, "sdp");

        let src = b"text/html; charset=ISO-8859-4\r\n";
        let mut scanner = Scanner::new(src);
        let c_type = ContentType::parse(&mut scanner);
        let c_type = c_type.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "text");
        assert_eq!(c_type.0.mimetype.subtype, "html");
        assert_eq!(c_type.0.param.unwrap().get("charset"), Some(&"ISO-8859-4"));
    }
}
