use crate::{
    error::Result,
    headers::SipHeaderParse,
    macros::{hdr_list, parse_header_param},
    parser::ParseCtx,
    MediaType,
};
use itertools::Itertools;
use std::fmt;

/// The `Accept` SIP header.
///
/// Indicates witch media types the client can process.
///
/// # Examples
///
/// ```
/// # use pksip::headers::Accept;
/// # use pksip::MediaType;
/// let mut accept = Accept::new();
///
/// accept.push(MediaType::new("application", "sdp"));
/// accept.push(MediaType::new("message", "sipfrag"));
///
/// assert_eq!(
///     accept.to_string(),
///     "Accept: application/sdp, message/sipfrag"
/// );
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Accept<'a>(Vec<MediaType<'a>>);

impl<'a> Accept<'a> {
    /// Creates a empty `Accept` header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an `MediaType` to the back of the header.
    #[inline]
    pub fn push(&mut self, mtype: MediaType<'a>) {
        self.0.push(mtype);
    }

    /// Returns a reference to an `MediaType` at the
    /// specified index.
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

impl<'a> SipHeaderParse<'a> for Accept<'a> {
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
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Accept<'a>> {
        let mtypes = hdr_list!(parser => {
            let mtype = parser.parse_token()?;
            parser.advance();
            let subtype = parser.parse_token()?;
            let param = parse_header_param!(parser);

            MediaType::from_parts(mtype, subtype, param)
        });

        Ok(Accept(mtypes))
    }
}

impl<'a> fmt::Display for Accept<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Accept::NAME, self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp;level=1, application/x-private, text/html\r\n";
        let mut scanner = ParseCtx::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(scanner.remaing(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(mtype.param.as_ref().unwrap().get("level").unwrap(), Some("1"));

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "x-private");

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "text");
        assert_eq!(mtype.mimetype.subtype, "html");

        let src = b"application/sdp, application/pidf+xml, message/sipfrag\r\n";
        let mut scanner = ParseCtx::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 3);
        assert_eq!(scanner.remaing(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "pidf+xml");

        let mtype = accept.get(2).unwrap();
        assert_eq!(mtype.mimetype.mtype, "message");
        assert_eq!(mtype.mimetype.subtype, "sipfrag");

        let src = b"application/sdp;q=0.8, application/simple-message-summary+xml;q=0.6\r\n";
        let mut scanner = ParseCtx::new(src);
        let accept = Accept::parse(&mut scanner).unwrap();

        assert!(accept.len() == 2);
        assert_eq!(scanner.remaing(), b"\r\n");

        let mtype = accept.get(0).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "sdp");
        assert_eq!(mtype.param.as_ref().unwrap().get("q").unwrap(), Some("0.8"));

        let mtype = accept.get(1).unwrap();
        assert_eq!(mtype.mimetype.mtype, "application");
        assert_eq!(mtype.mimetype.subtype, "simple-message-summary+xml");
        assert_eq!(mtype.param.as_ref().unwrap().get("q").unwrap(), Some("0.6"));
    }
}
