use core::str;

use crate::{
    scanner::Scanner,
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::{is_token, is_uri_content, Param, Result},
    uri::Params,
};

use super::SipHeaderParser;

pub struct GenericUri<'a> {
    scheme: &'a str,
    content: &'a str,
}

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
        let content = read_while!(scanner, is_uri_content);
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
