use super::{Header, ParseHeaderError};
use crate::{
    headers::{SipHeader, Q_PARAM},
    internal::{ArcStr, Q},
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::{self, Result},
};
use itertools::Itertools;
use reader::{util::is_newline, Reader};
use std::{fmt, result, str};

/// The `Accept-Encoding` SIP header.
///
/// Indicates what types of content encoding (compression) the client can process.
///
/// # Examples
///
/// ```
/// # use sip::{headers::{AcceptEncoding, accept_encoding::Coding}};
/// let mut encoding = AcceptEncoding::new();
///
/// encoding.push(Coding::new("gzip"));
/// encoding.push(Coding::new("compress"));
///
/// assert_eq!("Accept-Encoding: gzip, compress".as_bytes().try_into(), Ok(encoding));
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptEncoding(Vec<Coding>);

impl AcceptEncoding {
    /// Creates a empty `AcceptEncoding`.
    ///
    /// The header will not allocate until `Codings` are pushed onto it.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an `Coding` to the back of the header.
    #[inline]
    pub fn push(&mut self, coding: Coding) {
        self.0.push(coding);
    }

    /// Returns a reference to an `Coding` at the specified index.
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

impl TryFrom<&[u8]> for AcceptEncoding {
    type Error = ParseHeaderError;

    fn try_from(value: &[u8]) -> result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_accept_encoding()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl<'a, const N: usize> From<[Coding; N]> for AcceptEncoding {
    fn from(value: [Coding; N]) -> Self {
        Self(Vec::from(value))
    }
}

impl SipHeader<'_> for AcceptEncoding {
    const NAME: &'static str = "Accept-Encoding";
    /*
     * Accept-Encoding  =  "Accept-Encoding" HCOLON
     *                      [ encoding *(COMMA encoding) ]
     * encoding         =  codings *(SEMI accept-param)
     * codings          =  content-coding / "*"
     * content-coding   =  token
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        if reader.peek().is_some_and(|b| is_newline(b)) {
            return Ok(AcceptEncoding::new());
        }
        let codings = hdr_list!(reader => {
            let coding = parser::parse_token(reader)?;
            let mut q_param = None;
            let param = parse_header_param!(reader, Q_PARAM = q_param);
            let q = q_param.map(|q| q.parse()).transpose()?;

            Coding { coding: coding.into(), q, param }
        });

        Ok(AcceptEncoding(codings))
    }
}

impl fmt::Display for AcceptEncoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

/// A `coding` that apear in `Accept-Encoding` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coding {
    coding: ArcStr,
    q: Option<Q>,
    param: Option<Params>,
}

impl Coding {
    /// Creates a new `Coding` instance.
    pub fn new(coding: &str) -> Self {
        Self {
            coding: coding.into(),
            q: None,
            param: None,
        }
    }

    pub fn from_parts(
        coding: &str,
        q: Option<Q>,
        param: Option<Params>,
    ) -> Self {
        Self {
            coding: coding.into(),
            q,
            param,
        }
    }
}

impl fmt::Display for Coding {
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
        let mut reader = Reader::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 2);
        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "compress".into());
        assert_eq!(coding.q, None);

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "gzip".into());
        assert_eq!(coding.q, None);

        let mut reader = Reader::new(b"*\r\n");
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "*".into());
        assert_eq!(coding.q, None);

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut reader = Reader::new(src);
        let accept_encoding = AcceptEncoding::parse(&mut reader);
        let accept_encoding = accept_encoding.unwrap();

        assert!(accept_encoding.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let coding = accept_encoding.get(0).unwrap();
        assert_eq!(coding.coding, "gzip".into());
        assert_eq!(coding.q, Some(Q(1, 0)));

        let coding = accept_encoding.get(1).unwrap();
        assert_eq!(coding.coding, "identity".into());
        assert_eq!(coding.q, Some(Q(0, 5)));

        let coding = accept_encoding.get(2).unwrap();
        assert_eq!(coding.coding, "*".into());
        assert_eq!(coding.q, Some(Q(0, 0)));
    }
}
