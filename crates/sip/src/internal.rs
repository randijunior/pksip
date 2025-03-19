use std::{
    fmt,
    ops::Deref,
    str::{self, FromStr},
    sync::Arc,
};

use reader::{space, Reader};

use crate::{message::Params, parser::SipParserError};

use crate::parser::{self, Result};

/// A parameter.
///
/// This struct represents a parameter in a SIP message, consisting of a name and an optional value.
///
/// # Examples
///
/// ```
/// use sip::internal::Param;
///
/// let param: Param = "param=value".try_into().unwrap();
///
/// assert_eq!(param.name, "param");
/// assert_eq!(param.value, Some("value"));
/// ```
pub struct Param<'a> {
    pub name: &'a str,
    pub value: Option<&'a str>,
}

impl<'a> TryFrom<&'a str> for Param<'a> {
    type Error = SipParserError;

    fn try_from(
        s: &'a str,
    ) -> std::result::Result<Self, Self::Error> {
        Self::parse(&mut Reader::new(s.as_bytes()))
    }
}

impl<'a> Param<'a> {
    pub unsafe fn parse_unchecked<F>(
        reader: &mut Reader<'a>,
        func: F,
    ) -> Result<Self>
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

        Ok(Param {
            name,
            value: Some(value),
        })
    }

    pub fn parse(reader: &mut Reader<'a>) -> Result<Self> {
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
/// use sip::internal::Q;
///
/// let q_value = "0.5".parse();
/// assert_eq!(q_value, Ok(Q(0, 5)));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Q(pub u8, pub u8);

impl Q {
    pub fn new(a: u8, b: u8) -> Self {
        Self(a, b)
    }
}
impl From<u8> for Q {
    fn from(value: u8) -> Self {
        Self(value, 0)
    }
}
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
        match s.rsplit_once('.') {
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MimeType {
    pub mtype: ArcStr,
    pub subtype: ArcStr,
}

/// The `media-type` that appears in `Accept` and `Content-Type` SIP headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaType {
    pub mimetype: MimeType,
    pub param: Option<Params>,
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let MediaType { mimetype, param } = self;
        write!(f, "{}/{}", mimetype.mtype, mimetype.subtype)?;
        if let Some(param) = &param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

impl MediaType {
    /// Constructs a `MediaType` from a type and a subtype.
    pub fn new(mtype: &str, subtype: &str) -> Self {
        Self {
            mimetype: MimeType {
                mtype: mtype.into(),
                subtype: subtype.into(),
            },
            param: None,
        }
    }

    /// Constructs a `MediaType` with an optional parameters.
    pub fn from_parts(
        mtype: &str,
        subtype: &str,
        param: Option<Params>,
    ) -> Self {
        Self {
            mimetype: MimeType {
                mtype: mtype.into(),
                subtype: subtype.into(),
            },
            param,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArcStr(Arc<str>);

impl ArcStr {
    pub fn from_option_str(value: Option<&str>) -> Option<Self> {
        value.map(|v| Self(v.into()))
    }
}

impl Deref for ArcStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for ArcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ArcStr {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}
