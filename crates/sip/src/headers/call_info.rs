use super::{Header, ParseHeaderError, SipHeader};
use crate::{
    internal::ArcStr, macros::parse_header_param, message::Params,
    parser::Result,
};
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
/// let mut info = CallInfo::new("http://www.example.com/alice/", None, None);
///
/// assert_eq!("Call-Info: <http://www.example.com/alice/>".as_bytes().try_into(), Ok(info));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct CallInfo {
    url: ArcStr,
    purpose: Option<ArcStr>,
    params: Option<Params>,
}

impl CallInfo {
    pub fn new(url: &str, purpose: Option<&str>, params: Option<Params>) -> Self {
        Self { url: url.into(), purpose: purpose.map(|p| p.into()), params }
    }
    /// Set the url for this header.
    pub fn set_url(&mut self, url: ArcStr) {
        self.url = url;
    }
}

impl SipHeader<'_> for CallInfo {
    const NAME: &'static str = "Call-Info";
    /*
     * Call-Info = "Call-Info" HCOLON info * (COMMA info)
     * info = LAQUOT absoluteURI RAQUOT * (SEMI info-param)
     * info-param = ("purpose" EQUAL ("icon" | "info" | "card" | token)) |
     *		        generic-param
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let mut purpose: Option<ArcStr> = None;
        // must be an '<'
        reader.must_read(b'<')?;
        let url = until!(reader, &b'>');
        // must be an '>'
        reader.must_read(b'>')?;
        let url = str::from_utf8(url)?;
        let params = parse_header_param!(reader, PURPOSE = purpose);

        Ok(CallInfo {
            url: url.into(),
            params,
            purpose,
        })
    }
}

impl TryFrom<&[u8]> for CallInfo {
    type Error = ParseHeaderError;

    fn try_from(
        value: &[u8],
    ) -> std::result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_call_info()
            .map_err(|_| ParseHeaderError)?)
    }
}

impl fmt::Display for CallInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.url)?;
        if let Some(purpose) = &self.purpose {
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
        assert_eq!(
            info.url,
            "http://wwww.example.com/alice/photo.jpg".into()
        );
        assert_eq!(info.purpose, Some("icon".into()));

        let src =
            b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut reader = Reader::new(src);
        let info = CallInfo::parse(&mut reader).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/".into());
        assert_eq!(info.purpose, Some("info".into()));
    }
}
