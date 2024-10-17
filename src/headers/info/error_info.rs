use core::str;

use crate::{
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::{is_token, is_uri, Param, Result},
    scanner::Scanner,
    uri::Params,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct GenericUri<'a> {
    scheme: &'a str,
    content: &'a str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ErrorUri<'a> {
    url: GenericUri<'a>,
    params: Option<Params<'a>>,
}

impl<'a> ErrorUri<'a> {
    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        // must be an '<'
        let Some(&b'<') = scanner.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let scheme = read_while!(scanner, is_token);
        let scheme = unsafe { str::from_utf8_unchecked(scheme) };
        let Some(&b':') = scanner.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let content = read_while!(scanner, is_uri);
        let content = unsafe { str::from_utf8_unchecked(content) };
        // must be an '>'
        let Some(&b'>') = scanner.next() else {
            return sip_parse_error!("Invalid uri!");
        };
        let params = parse_param!(scanner, |param: Param<'a>| Some(param));

        Ok(ErrorUri {
            url: GenericUri { scheme, content },
            params,
        })
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ErrorInfo<'a>(Vec<ErrorUri<'a>>);

impl<'a> SipHeaderParser<'a> for ErrorInfo<'a> {
    const NAME: &'static [u8] = b"Error-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut infos: Vec<ErrorUri> = Vec::new();
        let uri = ErrorUri::parse(scanner)?;
        infos.push(uri);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            let uri = ErrorUri::parse(scanner)?;
            infos.push(uri);
            space!(scanner);
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
        let mut scanner = Scanner::new(src);
        let err_info = ErrorInfo::parse(&mut scanner).unwrap();
        assert_eq!(scanner.as_ref(), b"\r\n");

        let err = err_info.0.get(0).unwrap();
        assert_eq!(err.url.scheme, "sip");
        assert_eq!(err.url.content, "not-in-service-recording@atlanta.com");
    }
}
