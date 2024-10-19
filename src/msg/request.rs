use crate::{
    headers::SipHeaders,
    macros::{alpha, newline, space},
    parser::{parse_sip_version_2_0, SipParserError},
    scanner::Scanner,
    uri::Uri,
};

use super::SipMethod;

#[derive(Debug, PartialEq, Eq)]
pub struct RequestLine<'a> {
    pub(crate) method: SipMethod<'a>,
    pub(crate) uri: Uri<'a>,
}

impl<'a> RequestLine<'a> {
    pub fn from_bytes(src: &'a [u8]) -> Result<Self, SipParserError> {
        let mut scanner = Scanner::new(src);

        Self::parse(&mut scanner)
    }

    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<Self, SipParserError> {
        let method = alpha!(scanner);
        let method = SipMethod::from(method);

        space!(scanner);
        let uri = Uri::parse(scanner, true)?;
        space!(scanner);

        parse_sip_version_2_0(scanner)?;
        newline!(scanner);

        Ok(RequestLine { method, uri })
    }
}

#[derive(Debug)]
pub struct SipRequest<'a> {
    pub(crate) req_line: RequestLine<'a>,
    pub(crate) headers: SipHeaders<'a>,
    pub(crate) body: Option<&'a [u8]>,
}

impl<'a> SipRequest<'a> {
    pub fn new(
        req_line: RequestLine<'a>,
        headers: SipHeaders<'a>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            body,
            req_line,
            headers,
        }
    }

    pub fn request_line(&self) -> &RequestLine {
        &self.req_line
    }
}