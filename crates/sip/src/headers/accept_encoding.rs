use core::fmt;
use std::str;

use itertools::Itertools;
use reader::{util::is_newline, Reader};

use crate::{
    headers::Q_PARAM,
    macros::{hdr_list, parse_header_param},
    msg::Params,
    parser::{self, Result},
};

use crate::headers::SipHeader;

use super::Q;

/// A `coding` that apear in `Accept-Encoding` header
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Coding<'a> {
    coding: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
}

impl fmt::Display for Coding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Coding { coding, q, param } = self;
        write!(f, "{}", coding)?;
        if let Some(q) = q {
            write!(f, ";q={}.{}", q.0, q.1)?;
        }
        if let Some(param) = param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

/// `Accept-Encoding` SIP header.
///
/// The `Accept-Encoding` indicates what types of content encoding (compression) the client can
/// process.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
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

    fn parse(reader: &mut Reader<'a>) -> Result<AcceptEncoding<'a>> {
        if reader.peek().is_some_and(|b| is_newline(b)) {
            return Ok(AcceptEncoding::default());
        }
        let codings = hdr_list!(reader => {
            let coding = parser::parse_token(reader)?;
            let mut q_param = None;
            let param = parse_header_param!(reader, Q_PARAM = q_param);
            let q = q_param.and_then(|q| Q::parse(q));

            Coding { coding, q, param }
        });

        Ok(AcceptEncoding(codings))
    }
}

impl fmt::Display for AcceptEncoding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"compress, gzip\r\n";
        let mut reader = Reader::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 2);
        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "compress");
        assert_eq!(coding.q, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, None);

        let mut reader = Reader::new(b"*\r\n");
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, None);

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut reader = Reader::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, Some(Q(1, 0)));

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "identity");
        assert_eq!(coding.q, Some(Q(0, 5)));

        let coding = accept_encoding.get(2).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, Some(Q(0, 0)));
    }
}
