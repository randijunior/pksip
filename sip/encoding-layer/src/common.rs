use std::{fmt, str::{self,FromStr}};

use reader::{space, Reader};

use crate::{message::Params, parser::SipParserError};

use crate::parser::{self, Result};


/// A parameter.
///
/// This struct represents a parameter in a SIP message, consisting of a name and an optional value.
///
/// # Examples
///
/// ```rust
/// use encoding_layer::common::Param;
/// use reader::Reader;
///
/// let mut reader = Reader::new(b"param=value");
/// let param = Param::parse(&mut reader).unwrap();
///
/// assert_eq!(param.0, "param");
/// assert_eq!(param.1, Some("value"));
/// ```
pub struct Param<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> Param<'a> {
    pub unsafe fn parse_unchecked<F>(
        reader: &mut Reader<'a>,
        func: F,
    ) -> Result<Param<'a>>
    where
        F: Fn(&u8) -> bool,
    {
        space!(reader);
        let name = unsafe { reader.read_as_str(&func) };
        let Some(&b'=') = reader.peek() else {
            return Ok(Param { name, value: None });
        };
        reader.next();
        let value = if let Some(&b'"') = reader.peek() {
            reader.next();
            let value = reader::until!(reader, &b'"');
            reader.next();

            str::from_utf8(value)?
        } else {
            unsafe { reader.read_as_str(func) }
        };

        return Ok(Param { name, value: Some(value) });
    }

    pub fn parse(reader: &mut Reader<'a>) -> Result<Param<'a>> {
        unsafe { Self::parse_unchecked(reader, parser::is_token) }
    }
}


/// Represents a quality value (q-value) used in SIP headers.
///
/// The `Q` struct provides a method to parse a string representation of a q-value
/// into a `Q` instance. The q-value is typically used to indicate the preference
/// of certain SIP headers.
///
/// # Example
///
/// ```
/// use encoding_layer::common::Q;
///
/// let q_value = "0.5".parse();
/// assert_eq!(q_value, Ok(Q(0, 5)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Q(pub u8, pub u8);

#[derive(Debug, PartialEq, Eq)]
pub struct ParseQError;

impl From<ParseQError> for SipParserError {
    fn from(value: ParseQError) -> Self {
        SipParserError {
            message: format!("{:?}", value),
        }
    }
}

impl FromStr for Q {
    type Err = ParseQError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.rsplit_once(".") {
            Some((a, b)) => {
                let a = a.parse().map_err(|_| ParseQError)?;
                let b = b.parse().map_err(|_| ParseQError)?;
                Ok(Q(a, b))
            }
            None => match s.parse() {
                Ok(n) => Ok(Q(n, 0)),
                Err(_) => Err(ParseQError),
            },
        }
    }
}

impl fmt::Display for Q {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ";q={}.{}", self.0, self.1)
    }
}

/// This type reprents an MIME type that indicates an content format.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}

/// The `media-type` that appears in `Accept` and `Content-Type` SIP headers.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}

impl fmt::Display for MediaType<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MediaType { mimetype, param } = self;
        write!(f, "{}/{}", mimetype.mtype, mimetype.subtype)?;
        if let Some(param) = &param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

impl<'a> MediaType<'a> {
    pub fn new(
        mtype: &'a str,
        subtype: &'a str,
        param: Option<Params<'a>>,
    ) -> Self {
        Self {
            mimetype: MimeType { mtype, subtype },
            param,
        }
    }
}
