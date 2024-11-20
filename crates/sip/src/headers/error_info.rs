use std::str;

use scanner::Scanner;

use crate::{
    macros::{parse_header_list, parse_header_param},
    parser::Result,
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

    fn parse(scanner: &mut Scanner<'a>) -> Result<ErrorInfo<'a>> {
        let infos = parse_header_list!(scanner => {
            // must be an '<'
            scanner.must_read(b'<')?;
            let scheme = Token::parse(scanner);
            scanner.must_read(b':')?;
            let content = unsafe { scanner.read_and_convert_to_str(is_uri) };
            // must be an '>'
            scanner.must_read(b'>')?;
            let params = parse_header_param!(scanner);
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
        let mut scanner = Scanner::new(src);
        let err_info = ErrorInfo::parse(&mut scanner).unwrap();
        assert_eq!(scanner.as_ref(), b"\r\n");

        let err = err_info.0.get(0).unwrap();
        assert_eq!(err.url.scheme, "sip");
        assert_eq!(err.url.content, "not-in-service-recording@atlanta.com");
    }
}
