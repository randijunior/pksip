use core::str;

use crate::{
    bytes::Bytes,
    headers::SipHeaderParser,
    macros::{parse_header_param, read_while},
    parser::{is_token, Result},
};

use super::accept::{MediaType, MimeType};


pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeaderParser<'a> for ContentType<'a> {
    const NAME: &'static [u8] = b"Content-Type";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"c");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mtype = read_while!(bytes, is_token);
        let mtype = unsafe { str::from_utf8_unchecked(mtype) };
        bytes.next();
        let sub = read_while!(bytes, is_token);
        let sub = unsafe { str::from_utf8_unchecked(sub) };
        let param = parse_header_param!(bytes);

        Ok(ContentType(MediaType {
            mimetype: MimeType {
                mtype,
                subtype: sub,
            },
            param,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp\r\n";
        let mut bytes = Bytes::new(src);
        let c_type = ContentType::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "application");
        assert_eq!(c_type.0.mimetype.subtype, "sdp");

        let src = b"text/html; charset=ISO-8859-4\r\n";
        let mut bytes = Bytes::new(src);
        let c_type = ContentType::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "text");
        assert_eq!(c_type.0.mimetype.subtype, "html");
        assert_eq!(
            c_type.0.param.unwrap().get("charset"),
            Some(&"ISO-8859-4")
        );
    }
}
