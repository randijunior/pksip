use super::{Header, ParseHeaderError, SipHeader};
use crate::{macros::parse_header_param, message::Params, parser::Result};
use reader::{until, Reader};
use std::{fmt, str};

const PURPOSE: &'static str = "purpose";

/// The `Call-Info` SIP header.
///
/// Provides aditional information aboute the caller or calle.
///
/// # Examples
///
/// ```
/// # use sip::headers::CallInfo;
/// let mut info = CallInfo::default();
/// info.set_url("http://www.example.com/alice/");
///
/// assert_eq!("Call-Info: <http://www.example.com/alice/>".as_bytes().try_into(), Ok(info));
/// ```
#[derive(Debug, PartialEq, Eq, Default)]
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> CallInfo<'a> {
    /// Set the url for this header.
    pub fn set_url(&mut self, url: &'a str) {
        self.url = url;
    }
}

impl<'a> SipHeader<'a> for CallInfo<'a> {
    const NAME: &'static str = "Call-Info";
    /*
     * Call-Info = "Call-Info" HCOLON info * (COMMA info)
     * info = LAQUOT absoluteURI RAQUOT * (SEMI info-param)
     * info-param = ("purpose" EQUAL ("icon" | "info" | "card" | token)) |
     *		        generic-param
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        reader.must_read(b'<')?;
        let url = until!(reader, &b'>');
        // must be an '>'
        reader.must_read(b'>')?;
        let url = str::from_utf8(url)?;
        let params = parse_header_param!(reader, PURPOSE = purpose);

        Ok(CallInfo {
            url,
            params,
            purpose,
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for CallInfo<'a> {
    type Error = ParseHeaderError;

    fn try_from(value: &'a [u8]) -> std::result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_call_info()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl fmt::Display for CallInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.url)?;
        if let Some(purpose) = self.purpose {
            write!(f, ";{}", purpose)?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> \
        ;purpose=icon\r\n";
        let mut reader = Reader::new(src);
        let info = CallInfo::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon"));

        let src = b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut reader = Reader::new(src);
        let info = CallInfo::parse(&mut reader).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info"));
    }
}
