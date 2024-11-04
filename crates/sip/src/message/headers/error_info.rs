use core::str;

use crate::{
    bytes::Bytes,
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::Result,
    token::is_token,
    uri::{is_uri, GenericUri, Params},
};

use crate::headers::SipHeader;

pub struct ErrorUri<'a> {
    url: GenericUri<'a>,
    params: Option<Params<'a>>,
}

impl<'a> ErrorUri<'a> {
    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        // must be an '<'
        let Some(&b'<') = bytes.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let scheme = read_while!(bytes, is_token);
        let scheme = unsafe { str::from_utf8_unchecked(scheme) };
        let Some(&b':') = bytes.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let content = read_while!(bytes, is_uri);
        let content = unsafe { str::from_utf8_unchecked(content) };
        // must be an '>'
        let Some(&b'>') = bytes.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let params = parse_param!(bytes);

        Ok(ErrorUri {
            url: GenericUri { scheme, content },
            params,
        })
    }
}

/// Provides a pointer to additional information about the error status response.
pub struct ErrorInfo<'a>(Vec<ErrorUri<'a>>);

impl<'a> SipHeader<'a> for ErrorInfo<'a> {
    const NAME: &'static str = "Error-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut infos: Vec<ErrorUri> = Vec::new();
        let uri = ErrorUri::parse(bytes)?;
        infos.push(uri);

        while let Some(b',') = bytes.peek() {
            bytes.next();
            let uri = ErrorUri::parse(bytes)?;
            infos.push(uri);
            space!(bytes);
        }

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
