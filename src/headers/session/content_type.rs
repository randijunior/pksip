use core::str;

use crate::{
    headers::SipHeaderParser,
    macros::{parse_header_param, read_while},
    parser::{is_token, Result},
    scanner::Scanner,
};

use super::accept::{MediaType, MimeType};

#[derive(Debug, PartialEq, Eq)]
pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeaderParser<'a> for ContentType<'a> {
    const NAME: &'static [u8] = b"Content-Type";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"c");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mtype = read_while!(scanner, is_token);
        let mtype = unsafe { str::from_utf8_unchecked(mtype) };
        scanner.next();
        let sub = read_while!(scanner, is_token);
        let sub = unsafe { str::from_utf8_unchecked(sub) };
        let param = parse_header_param!(scanner);

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
    use std::collections::HashMap;

    use crate::uri::Params;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp\r\n";
        let mut scanner = Scanner::new(src);
        let c_type = ContentType::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "application");
        assert_eq!(c_type.0.mimetype.subtype, "sdp");

        let src = b"text/html; charset=ISO-8859-4\r\n";
        let mut scanner = Scanner::new(src);
        let c_type = ContentType::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_type.0.mimetype.mtype, "text");
        assert_eq!(c_type.0.mimetype.subtype, "html");
        assert_eq!(
            c_type.0.param,
            Some(Params::from(HashMap::from([(
                "charset",
                Some("ISO-8859-4")
            )])))
        );
    }
}
