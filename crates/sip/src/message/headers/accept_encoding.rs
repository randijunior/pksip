use std::str;

use crate::{
    bytes::Bytes,
    headers::{self, Q_PARAM},
    macros::{parse_comma_separated_header, parse_param},
    parser::{self, Result},
    uri::Params,
    util::is_newline,
};

use crate::headers::SipHeader;

/// A `coding` that apear in `Accept-Encoding` header
#[derive(Default)]
pub struct Coding<'a> {
    coding: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}


/// A `Accept-Encoding` SIP header.
///
/// The `Accept-Encoding` indicates what types of content encoding (compression) the client can
/// process.
#[derive(Default)]
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> AcceptEncoding<'a> {
    pub fn get(&self, index: usize) -> Option<&Coding<'a>> {
        self.0.get(index)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeader<'a> for AcceptEncoding<'a> {
    const NAME: &'static str = "Accept-Encoding";

    fn parse(bytes: &mut Bytes<'a>) -> Result<AcceptEncoding<'a>> {
        if bytes.peek().is_some_and(|b| is_newline(b)) {
            return Ok(AcceptEncoding::default());
        }
        let codings = parse_comma_separated_header!(bytes => {
            let coding = parser::parse_token(bytes);
            let mut q_param = None;
            let param = parse_param!(bytes, Q_PARAM = q_param);
            let q = q_param.and_then(|q| headers::parse_q(Some(q)));
    
            Coding { coding, q, param }
        });

        Ok(AcceptEncoding(codings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"compress, gzip\r\n";
        let mut bytes = Bytes::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut bytes).unwrap();

        assert!(accept_encoding.len() == 2);
        assert_eq!(bytes.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "compress");
        assert_eq!(coding.q, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, None);

        let mut bytes = Bytes::new(b"*\r\n");
        let accept_encoding = AcceptEncoding::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, None);

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut bytes = Bytes::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut bytes).unwrap();

        assert!(accept_encoding.len() == 3);
        assert_eq!(bytes.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, Some(1.0));

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "identity");
        assert_eq!(coding.q, Some(0.5));

        let coding = accept_encoding.get(2).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, Some(0.0));
    }
}
