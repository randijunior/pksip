//! SIP Request Types
//!
//! The module provide the [`SipRequest`]

use crate::{
    bytes::Bytes,
    macros::{alpha, newline, space},
    parser::{self, SipParserError},
    uri::Uri,
};

use super::{headers::Headers, SipMethod};

/// Represents an SIP Request-Line

pub struct RequestLine<'a> {
    pub(crate) method: SipMethod<'a>,
    pub(crate) uri: Uri<'a>,
}

impl<'a> RequestLine<'a> {
    pub fn from_bytes(src: &'a [u8]) -> Result<Self, SipParserError> {
        let mut bytes = Bytes::new(src);

        Self::parse(&mut bytes)
    }

    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self, SipParserError> {
        let method = alpha!(bytes);
        let method = SipMethod::from(method);

        space!(bytes);
        let uri = Uri::parse(bytes, true)?;
        space!(bytes);

        parser::parse_sip_v2(bytes)?;
        newline!(bytes);

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
