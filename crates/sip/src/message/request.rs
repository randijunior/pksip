//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use reader::{alpha, newline, space, Reader};

use crate::{
    headers::Headers,
    parser::{SipParser, SipParserError},
    uri::Uri,
};

use super::SipMethod;

/// Represents an SIP Request-Line

pub struct RequestLine<'a> {
    pub(crate) method: SipMethod<'a>,
    pub(crate) uri: Uri<'a>,
}

impl<'a> RequestLine<'a> {
    pub fn from_bytes(src: &'a [u8]) -> Result<Self, SipParserError> {
        let mut reader = Reader::new(src);

        Self::parse(&mut reader)
    }

    pub(crate) fn parse(
        reader: &mut Reader<'a>,
    ) -> Result<Self, SipParserError> {
        let method = alpha!(reader);
        let method = SipMethod::from(method);

        space!(reader);
        let uri = Uri::parse(reader, true)?;
        space!(reader);

        SipParser::parse_sip_v2(reader)?;
        newline!(reader);

        Ok(RequestLine { method, uri })
    }
}

pub struct SipRequest<'a> {
    pub(crate) req_line: RequestLine<'a>,
    pub(crate) headers: Headers<'a>,
    pub(crate) body: Option<&'a [u8]>,
}

impl<'a> SipRequest<'a> {
    pub fn new(
        req_line: RequestLine<'a>,
        headers: Headers<'a>,
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
