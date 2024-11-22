use std::str;

use reader::Reader;

use crate::{
    macros::{parse_header_list, parse_header_param},
    parser::Result,
    token::Token,
    uri::{is_uri, GenericUri, Params},
};

use crate::headers::SipHeader;
#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfoUri<'a> {
    url: GenericUri<'a>,
    params: Option<Params<'a>>,
}

/// The `Error-Info` SIP header.
///
/// Provides a pointer to additional information about the error status response.
#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfo<'a>(Vec<ErrorInfoUri<'a>>);

impl<'a> SipHeader<'a> for ErrorInfo<'a> {
    const NAME: &'static str = "Error-Info";

    fn parse(reader: &mut Reader<'a>) -> Result<ErrorInfo<'a>> {
        let infos = parse_header_list!(reader => {
            // must be an '<'
            reader.must_read(b'<')?;
            let scheme = Token::parse(reader);
            reader.must_read(b':')?;
            let content = unsafe { reader.read_while_as_str(is_uri) };
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
