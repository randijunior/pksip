use super::{Header, ParseHeaderError};
use crate::{
    headers::{SipHeader, Q_PARAM},
    internal::Q,
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::{self, Result},
};
use itertools::Itertools;
use reader::{util::is_newline, Reader};
use std::{fmt, str};

/// The `Accept-Encoding` SIP header.
///
/// Indicates what types of content encoding (compression) the client can process.
///
/// # Examples
///
/// ```
/// # use sip::{headers::{AcceptEncoding, accept_encoding::Coding}};
/// # use sip::internal::Q;
/// let mut encoding = AcceptEncoding::new();
///
/// encoding.push(Coding::new("gzip", Some(Q::from(1)), None));
/// encoding.push(Coding::new("compress", None, None));
///
/// assert_eq!("Accept-Encoding: gzip;q=1, compress".as_bytes().try_into(), Ok(encoding));
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> AcceptEncoding<'a> {
    /// Creates a empty `AcceptEncoding` header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an new `Coding` at the end of the header.
    pub fn push(&mut self, coding: Coding<'a>) {
        self.0.push(coding);
    }

    /// Gets the `Coding` at the specified index.
    pub fn get(&self, index: usize) -> Option<&Coding<'a>> {
        self.0.get(index)
    }

    /// Returns the number of `Codings` in the header.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for AcceptEncoding<'a> {
    type Error = ParseHeaderError;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_accept_encoding()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl<'a> SipHeader<'a> for AcceptEncoding<'a> {
    const NAME: &'static str = "Accept-Encoding";
    /*
     * Accept-Encoding  =  "Accept-Encoding" HCOLON
     *                      [ encoding *(COMMA encoding) ]
     * encoding         =  codings *(SEMI accept-param)
     * codings          =  content-coding / "*"
     * content-coding   =  token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        if reader.peek().is_some_and(|b| is_newline(b)) {
            return Ok(AcceptEncoding::new());
        }
        let codings = hdr_list!(reader => {
            let coding = parser::parse_token(reader)?;
            let mut q_param = None;
            let param = parse_header_param!(reader, Q_PARAM = q_param);
            let q = q_param.map(|q| q.parse()).transpose()?;

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

/// A `coding` that apear in `Accept-Encoding` header.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Coding<'a> {
    coding: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    /// Creates a new `Coding` instance.
    pub fn new(
        coding: &'a str,
        q: Option<Q>,
        param: Option<Params<'a>>,
    ) -> Self {
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
