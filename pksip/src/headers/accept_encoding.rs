use crate::{
    error::Result,
    headers::{SipHeaderParse, Q_PARAM},
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::Parser,
    Q,
};
use itertools::Itertools;
use std::{fmt, str};

/// The `Accept-Encoding` SIP header.
///
/// Indicates what types of content encoding (compression)
/// the client can process.
///
/// # Examples
///
/// ```
/// # use pksip::{headers::{AcceptEncoding, Coding}};
/// let mut encoding = AcceptEncoding::new();
///
/// encoding.push(Coding::new("gzip"));
/// encoding.push(Coding::new("compress"));
///
/// assert_eq!(
///     encoding.to_string(),
///     "Accept-Encoding: gzip, compress",
/// );
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> AcceptEncoding<'a> {
    /// Creates a empty `AcceptEncoding`.
    ///
    /// The header will not allocate until `Codings` are
    /// pushed onto it.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an `Coding` to the back of the header.
    #[inline]
    pub fn push(&mut self, coding: Coding<'a>) {
        self.0.push(coding);
    }

    /// Returns a reference to an `Coding` at the specified
    /// index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Coding> {
        self.0.get(index)
    }

    /// Returns the number of elements in the header.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, const N: usize> From<[Coding<'a>; N]> for AcceptEncoding<'a> {
    fn from(value: [Coding<'a>; N]) -> Self {
        Self(Vec::from(value))
    }
}

impl<'a> SipHeaderParse<'a> for AcceptEncoding<'a> {
    const NAME: &'static str = "Accept-Encoding";
    /*
     * Accept-Encoding  =  "Accept-Encoding" HCOLON
     *                      [ encoding *(COMMA encoding) ]
     * encoding         =  codings *(SEMI accept-param)
     * codings          =  content-coding / "*"
     * content-coding   =  token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        if parser.is_next_newline() {
            return Ok(AcceptEncoding::new());
        }
        let codings = hdr_list!(parser => {
            let coding = parser.parse_token()?;
            let mut q_param = None;
            let param = parse_header_param!(parser, Q_PARAM = q_param);
            let q = q_param.map(|q| q.parse()).transpose()?;

            Coding { coding, q, param }
        });

        Ok(AcceptEncoding(codings))
    }
}

impl fmt::Display for AcceptEncoding<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", AcceptEncoding::NAME, self.0.iter().format(", "))
    }
}

/// A `coding` that apear in `Accept-Encoding` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coding<'a> {
    coding: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    /// Creates a new `Coding` instance.
    pub fn new(coding: &'a str) -> Self {
        Self {
            coding,
            q: None,
            param: None,
        }
    }

    /// Creates a new `Coding` header with the given coding, q param and another params.
    pub fn from_parts(coding: &'a str, q: Option<Q>, param: Option<Params<'a>>) -> Self {
        Self { coding, q, param }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"compress, gzip\r\n";
        let mut parser = Parser::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut parser);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 2);
        assert_eq!(parser.remaing(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "compress");
        assert_eq!(coding.q, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "gzip");
        assert_eq!(coding.q, None);

        let mut parser = Parser::new(b"*\r\n");
        let accept_encoding = AcceptEncoding::parse(&mut parser);
        let accept_encoding = accept_encoding.unwrap();

        assert_eq!(parser.remaing(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "*");
        assert_eq!(coding.q, None);

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut parser = Parser::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut parser);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 3);
        assert_eq!(parser.remaing(), b"\r\n");

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
