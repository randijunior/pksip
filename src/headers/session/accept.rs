use core::str;

use crate::{
    macros::{parse_param, read_until_byte, read_while, sip_parse_error, space},
    parser::Result,
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

use crate::headers::SipHeaderParser;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}
#[derive(Debug, PartialEq, Eq)]
pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}
#[derive(Debug)]
pub struct Accept<'a>(Vec<MediaType<'a>>);

impl<'a> Accept<'a> {
    pub fn get(&self, index: usize) -> Option<&MediaType<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for Accept<'a> {
    const NAME: &'static [u8] = b"Accept";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Accept<'a>> {
        let mut mtypes: Vec<MediaType<'a>> = Vec::new();
        loop {
            let is_next_newline = scanner.peek().is_some_and(|&c| is_newline(c));
            if scanner.is_eof() || is_next_newline {
                break;
            }
            let mtype = read_until_byte!(scanner, b'/');
            scanner.next();
            let subtype = read_while!(scanner, |c: u8| c != b',' && !is_newline(c) && c != b';');

            let param = parse_param!(scanner, |param| Some(param));
            let media_type = MediaType {
                mimetype: MimeType {
                    mtype: str::from_utf8(mtype)?,
                    subtype: str::from_utf8(subtype)?,
                },
                param,
            };
            mtypes.push(media_type);
            scanner.read_if_eq(b',')?;
            space!(scanner);
        }

        Ok(Accept(mtypes))
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp;level=1, application/x-private, text/html\r\n";
        let mut scanner = Scanner::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(
            mtype.param,
            Some(Params::from(HashMap::from([("level", Some("1"))])))
        );

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "x-private");
        assert_eq!(mtype.param, None);

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "text");
        assert_eq!(mtype.mimetype.subtype, "html");
        assert_eq!(mtype.param, None);

        let src = b"application/sdp, application/pidf+xml, message/sipfrag\r\n";
        let mut scanner = Scanner::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(mtype.param, None);

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "pidf+xml");
        assert_eq!(mtype.param, None);

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "message");
        assert_eq!(mtype.mimetype.subtype, "sipfrag");
        assert_eq!(mtype.param, None);

        let src = b"application/sdp;q=0.8, application/simple-message-summary+xml;q=0.6\r\n";
        let mut scanner = Scanner::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 2);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(
            mtype.param,
            Some(Params::from(HashMap::from([("q", Some("0.8"))])))
        );

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "simple-message-summary+xml");
        assert_eq!(
            mtype.param,
            Some(Params::from(HashMap::from([("q", Some("0.6"))])))
        );
    }
}