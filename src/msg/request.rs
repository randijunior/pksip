use crate::{
    headers::SipHeaders,
    parser::{parse_request_line, SipParserError},
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
    pub fn from_bytes(src: &[u8]) -> Result<RequestLine, SipParserError> {
        let mut scanner = Scanner::new(src);

        parse_request_line(&mut scanner)
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
