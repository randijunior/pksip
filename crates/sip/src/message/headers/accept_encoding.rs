use std::str;

use crate::{
    bytes::Bytes,
    headers::{self, Q_PARAM},
    macros::{parse_header_param, space},
    parser::{self, is_token, Result},
    uri::Params,
    util::is_newline,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Default)]
pub struct Coding<'a> {
    coding: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        space!(bytes);
        let coding = parser::parse_token(bytes);
        let mut q_param = None;
        let param = parse_header_param!(bytes, Q_PARAM = q_param);
        let q = q_param.and_then(|q| headers::parse_q(Some(q)));

        Ok(Coding { coding, q, param })
    }
}

// Accept-Encoding  =  "Accept-Encoding" HCOLON
//                     [ encoding *(COMMA encoding) ]
// encoding         =  codings *(SEMI accept-param)
// codings          =  content-coding / "*"
// content-coding   =  token
#[derive(Debug, PartialEq, Default)]
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> AcceptEncoding<'a> {
    pub fn get(&self, index: usize) -> Option<&Coding<'a>> {
        self.0.get(index)
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for AcceptEncoding<'a> {
    const NAME: &'static [u8] = b"Accept-Encoding";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        space!(bytes);

        if bytes.peek().is_some_and(|b| is_newline(b)) {
            return Ok(AcceptEncoding::default());
        }
        let mut codings: Vec<Coding> = Vec::new();
        let coding = Coding::parse(bytes)?;
        codings.push(coding);

        while let Some(b',') = bytes.peek() {
            bytes.next();
            let coding = Coding::parse(bytes)?;
            codings.push(coding);
        }

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
        assert_eq!(coding.param, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, None);
        assert_eq!(coding.param, None);

        let mut bytes = Bytes::new(b"*\r\n");
        let accept_encoding = AcceptEncoding::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, None);
        assert_eq!(coding.param, None);

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut bytes = Bytes::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut bytes).unwrap();

        assert!(accept_encoding.len() == 3);
        assert_eq!(bytes.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, Some(1.0));
        assert_eq!(coding.param, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "identity");
        assert_eq!(coding.q, Some(0.5));
        assert_eq!(coding.param, None);

        let coding = accept_encoding.get(2).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, Some(0.0));
        assert_eq!(coding.param, None);
    }
}
