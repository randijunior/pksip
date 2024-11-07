use core::str;

use crate::{
    bytes::Bytes,
    macros::{parse_header_list, parse_param},
    parser::{self, Result},
    token::Token,
    uri::{is_uri, GenericUri, Params},
};

use crate::headers::SipHeader;

pub struct ErrorInfoUri<'a> {
    url: GenericUri<'a>,
    params: Option<Params<'a>>,
}

/// The `Error-Info` SIP header.
///
/// Provides a pointer to additional information about the error status response.
pub struct ErrorInfo<'a>(Vec<ErrorInfoUri<'a>>);

impl<'a> SipHeader<'a> for ErrorInfo<'a> {
    const NAME: &'static str = "Error-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<ErrorInfo<'a>> {
        let infos = parse_header_list!(bytes => {
            // must be an '<'
            bytes.must_read(b'<')?;
            let scheme = Token::parse(bytes);
            bytes.must_read(b':')?;
            let content = unsafe { parser::extract_as_str(bytes, is_uri) };
            // must be an '>'
            bytes.must_read(b'>')?;
            let params = parse_param!(bytes);
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
        let mut bytes = Bytes::new(src);
        let err_info = ErrorInfo::parse(&mut bytes).unwrap();
        assert_eq!(bytes.as_ref(), b"\r\n");

        let err = err_info.0.get(0).unwrap();
        assert_eq!(err.url.scheme, "sip");
        assert_eq!(err.url.content, "not-in-service-recording@atlanta.com");
    }
}
