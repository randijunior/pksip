use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::{
    macros::{hdr_list, parse_header_param},
    msg::{GenericUri, Params},
    parser::{self, is_uri, Result},
};

use crate::headers::SipHeader;
#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfoUri<'a> {
    url: GenericUri<'a>,
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

    fn parse(reader: &mut Reader<'a>) -> Result<ErrorInfo<'a>> {
        let infos = hdr_list!(reader => {
            // must be an '<'
            reader.must_read(b'<')?;
            let scheme = parser::parse_token(reader)?;
            reader.must_read(b':')?;
            let content = unsafe { reader.read_as_str(is_uri) };
            // must be an '>'
            reader.must_read(b'>')?;
            let params = parse_header_param!(reader);
            ErrorInfoUri {
                url: GenericUri { scheme, content },
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
        assert_eq!(err.url.scheme, "sip");
        assert_eq!(err.url.content, "not-in-service-recording@atlanta.com");
    }
}
