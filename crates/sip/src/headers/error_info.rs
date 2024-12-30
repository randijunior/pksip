use std::{fmt, str};

use itertools::Itertools;
use reader::{until, Reader};

use crate::{
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::Result,
};

use crate::headers::SipHeader;
#[derive(Debug, PartialEq, Eq)]
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
/// Provides a pointer to additional information about the error status response.
#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfo<'a>(Vec<ErrorInfoUri<'a>>);

impl<'a> SipHeader<'a> for ErrorInfo<'a> {
    const NAME: &'static str = "Error-Info";
    /*
     * Error-Info  =  "Error-Info" HCOLON error-uri *(COMMA error-uri)
     * error-uri   =  LAQUOT absoluteURI RAQUOT *( SEMI generic-param )
     */
    fn parse(reader: &mut Reader<'a>) -> Result<ErrorInfo<'a>> {
        let infos = hdr_list!(reader => {
            reader.must_read(b'<')?;
            let url = until!(reader, &b'>');
            reader.must_read(b'>')?;

            let url = str::from_utf8(url)?;
            let params = parse_header_param!(reader);
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
        let mut reader = Reader::new(src);
        let err_info = ErrorInfo::parse(&mut reader).unwrap();
        assert_eq!(reader.as_ref(), b"\r\n");

        let err = err_info.0.get(0).unwrap();
        assert_eq!(err.url, "sip:not-in-service-recording@atlanta.com");
    }
}
