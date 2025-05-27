use std::{fmt, str};

use itertools::Itertools;

use crate::{
    error::Result,
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::ParseCtx,
};

use crate::headers::SipHeaderParse;
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorInfoUri<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

impl fmt::Display for ErrorInfoUri<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)?;

        if let Some(param) = &self.params {
            write!(f, ";{}", param)?;
        }

        Ok(())
    }
}

/// The `Error-Info` SIP header.
///
/// Provides a pointer to additional information about the
/// error status response.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ErrorInfo<'a>(Vec<ErrorInfoUri<'a>>);

impl<'a> SipHeaderParse<'a> for ErrorInfo<'a> {
    const NAME: &'static str = "Error-Info";
    /*
     * Error-Info  =  "Error-Info" HCOLON error-uri *(COMMA error-uri)
     * error-uri   =  LAQUOT absoluteURI RAQUOT *( SEMI generic-param )
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let infos = hdr_list!(parser => {
            parser.advance();
            let url = parser.read_until_byte(b'>');
            parser.advance();

            let url = str::from_utf8(url)?;
            let params = parse_header_param!(parser);
            ErrorInfoUri {
                url,
                params
            }
        });

        Ok(ErrorInfo(infos))
    }
}

impl fmt::Display for ErrorInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:not-in-service-recording@atlanta.com>\r\n";
        let mut scanner = ParseCtx::new(src);
        let err_info = ErrorInfo::parse(&mut scanner).unwrap();
        assert_eq!(scanner.remaing(), b"\r\n");

        let err = err_info.0.get(0).unwrap();
        assert_eq!(err.url, "sip:not-in-service-recording@atlanta.com");
    }
}
