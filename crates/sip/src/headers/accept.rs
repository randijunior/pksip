use super::{Header, ParseHeaderError};
use crate::{
    headers::SipHeader,
    internal::MediaType,
    macros::{hdr_list, parse_header_param},
    parser::{self, Result},
};
use itertools::Itertools;
use reader::Reader;
use std::{fmt, result};

/// The `Accept` SIP header.
///
/// Indicates witch media types the client can process.
///
/// # Examples
///
/// ```
/// # use sip::headers::Accept;
/// # use sip::internal::MediaType;
/// let mut accept = Accept::new();
///
/// accept.push(MediaType::new("application", "sdp"));
/// accept.push(MediaType::new("message", "sipfrag"));
///
/// assert_eq!("Accept: application/sdp, message/sipfrag".as_bytes().try_into(), Ok(accept));
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Accept(Vec<MediaType>);

impl Accept {
    /// Creates a empty `Accept` header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an `MediaType` to the back of the header.
    #[inline]
    pub fn push(&mut self, mtype: MediaType) {
        self.0.push(mtype);
    }

    /// Returns a reference to an `MediaType` at the specified index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&MediaType> {
        self.0.get(index)
    }

    /// Returns the number of elements in the header.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> TryFrom<&'a [u8]> for Accept {
    type Error = ParseHeaderError;

    fn try_from(value: &'a [u8]) -> result::Result<Self, Self::Error> {
        Ok(Header::from_bytes(value)?
            .into_accept()
            .map_err(|_| ParseHeaderError(Self::NAME))?)
    }
}

impl SipHeader<'_> for Accept {
    const NAME: &'static str = "Accept";
    /*
     * Accept         =  "Accept" HCOLON [ accept-range *(COMMA accept-range) ]
     * accept-range   =  media-range *(SEMI accept-param)
     * media-range =  ( "*" "/" "*"
     *                / ( m-type SLASH "*" )
     *                / ( m-type SLASH m-subtype )
     *                ) *( SEMI m-parameter )
     * accept-param   =  ("q" EQUAL qvalue) / generic-param
     * qvalue         =  ( "0" [ "." 0*3DIGIT ] ) ( "1" [ "." 0*3("0") ] )
     * generic-param  =  token [ EQUAL gen-value ]
     * gen-value      =  token / host / quoted-string
     */
    fn parse(reader: &mut Reader) -> Result<Accept> {
        let mtypes = hdr_list!(reader => {
            let mtype = parser::parse_token(reader)?;
            reader.must_read(&b'/')?;
            let subtype = parser::parse_token(reader)?;
            let param = parse_header_param!(reader);

            MediaType::from_parts(mtype, subtype, param)
        });

        Ok(Accept(mtypes))
    }
}

impl fmt::Display for Accept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src =
            b"application/sdp;level=1, application/x-private, text/html\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "sdp".into());
        assert_eq!(
            mtype.param.as_ref().unwrap().get("level".into()),
            Some("1")
        );

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "x-private".into());

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "text".into());
        assert_eq!(mtype.mimetype.subtype, "html".into());

        let src = b"application/sdp, application/pidf+xml, message/sipfrag\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "sdp".into());

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "pidf+xml".into());

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "message".into());
        assert_eq!(mtype.mimetype.subtype, "sipfrag".into());

        let src = b"application/sdp;q=0.8, application/simple-message-summary+xml;q=0.6\r\n";
        let mut reader = Reader::new(src);
        let accept = Accept::parse(&mut reader).unwrap();

        assert!(accept.len() == 2);
        assert_eq!(reader.as_ref(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "sdp".into());
        assert_eq!(mtype.param.as_ref().unwrap().get("q".into()), Some("0.8"));

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application".into());
        assert_eq!(mtype.mimetype.subtype, "simple-message-summary+xml".into());
        assert_eq!(mtype.param.as_ref().unwrap().get("q".into()), Some("0.6"));
    }
}
