use std::str;

use reader::Reader;

use crate::{
    macros::{parse_header_list, parse_header_param},
    parser::Result,
    token::Token,
};

use crate::headers::SipHeader;

use super::MediaType;

/// The `Accept` SIP header.
///
/// Indicates witch media types the client can process.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Accept<'a>(Vec<MediaType<'a>>);

impl<'a> Accept<'a> {
    pub fn get(&self, index: usize) -> Option<&MediaType<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeader<'a> for Accept<'a> {
    const NAME: &'static str = "Accept";

    fn parse(reader: &mut Reader<'a>) -> Result<Accept<'a>> {
        let mtypes = parse_header_list!(reader => {
            let mtype = Token::parse(reader);
            reader.must_read(b'/')?;
            let subtype = Token::parse(reader);

            let param = parse_header_param!(reader);

            MediaType::new(mtype, subtype, param)
        });

        Ok(Accept(mtypes))
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src =
            b"application/sdp;level=1, application/x-private, text/html\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(mtype.param.as_ref().unwrap().get("level"), Some(&"1"));

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "x-private");

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "text");
        assert_eq!(mtype.mimetype.subtype, "html");

        let src = b"application/sdp, application/pidf+xml, message/sipfrag\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "pidf+xml");

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "message");
        assert_eq!(mtype.mimetype.subtype, "sipfrag");

        let src = b"application/sdp;q=0.8, application/simple-message-summary+xml;q=0.6\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 2);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(mtype.param.as_ref().unwrap().get("q"), Some(&"0.8"));

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "simple-message-summary+xml");
        assert_eq!(mtype.param.as_ref().unwrap().get("q"), Some(&"0.6"));
    }
}
