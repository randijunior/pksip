use super::{Header, ParseHeaderError, SipHeader};
use crate::{
    internal::ArcStr,
    macros::parse_header_param,
    message::Params,
    parser::{self, Result},
};
use core::fmt;
use reader::Reader;

/// The `Content-Disposition` SIP header.
///
/// Describes how the `message-body` is to be interpreted by the `UAC` or `UAS`.
///
/// # Examples
///
/// ```
/// # use sip::headers::content_disposition::ContentDisposition;
/// let c_disp = ContentDisposition::new("session".into(), None);
///
/// assert_eq!("Content-Disposition: session".as_bytes().try_into(), Ok(c_disp));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct ContentDisposition {
    _type: ArcStr,
    params: Option<Params>,
}

impl ContentDisposition {
    /// Creates a new `ContentDisposition` instance.
    pub fn new(_type: ArcStr, params: Option<Params>) -> Self {
        Self { _type, params }
    }
}

impl SipHeader<'_> for ContentDisposition {
    const NAME: &'static str = "Content-Disposition";
    /*
     * Content-Disposition   =  "Content-Disposition" HCOLON
     *                          disp-type *( SEMI disp-param )
     * disp-type             =  "render" / "session" / "icon" / "alert"
     *                          / disp-extension-token
     * disp-param            =  handling-param / generic-param
     * handling-param        =  "handling" EQUAL
     *                          ( "optional" / "required"
     *                          / other-handling )
     * other-handling        =  token
     * disp-extension-token  =  token
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let _type = parser::parse_token(reader)?;
        let params = parse_header_param!(reader);

        Ok(ContentDisposition {
            _type: _type.into(),
            params,
        })
    }
}

impl TryFrom<&[u8]> for ContentDisposition {
    type Error = ParseHeaderError;

    fn try_from(
        value: &[u8],
    ) -> std::result::Result<Self, Self::Error> {
        Header::from_bytes(value)?
            .into_content_disposition()
            .map_err(|_| ParseHeaderError(Self::NAME))
    }
}

impl fmt::Display for ContentDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self._type)?;

        if let Some(param) = &self.params {
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
        let src = b"session\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session".into());

        let src = b"session;handling=optional\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session".into());
        assert_eq!(
            disp.params.unwrap().get("handling".into()),
            Some("optional")
        );

        let src =
            b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut reader = Reader::new(src);
        let disp = ContentDisposition::parse(&mut reader);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "attachment".into());
        let params = disp.params.unwrap();

        assert_eq!(params.get("filename".into()), Some("smime.p7s"));
        assert_eq!(params.get("handling".into()), Some("required"));
    }
}
