//! SIP Parser

use crate::bytes::Bytes;
use crate::macros::read_while;
use crate::macros::sip_parse_error;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use std::str;
use std::num::ParseFloatError;
use std::num::ParseIntError;
use std::str::Utf8Error;

use crate::bytes::BytesError;
use crate::message::headers::Headers;

use crate::macros::peek_while;

use crate::message::SipMessage;
use crate::message::StatusLine;
use crate::message::{RequestLine, SipRequest, SipResponse};

use crate::util::is_alphabetic;

pub(crate) const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

pub(crate) const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                                    ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                    0123456789";

pub(crate) const UNRESERVED: &[u8] = b"-_.!~*'()%";
pub(crate) const ESCAPED: &[u8] = b"%";
pub(crate) const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
pub(crate) const TOKEN: &[u8] = b"-.!%*_`'~+";
pub(crate) const PASS: &[u8] = b"&=+$,";
pub(crate) const HOST: &[u8] = b"_-.";
pub(crate) const GENERIC_URI: &[u8] = b"#?;:@&=+-_.!~*'()%$,/";

pub struct SipParser;

impl SipParser {
    /// Parse a buff of bytes into sip message
    pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
        let mut bytes = Bytes::new(buff);

        let msg = if !Self::is_sip_version(&bytes) {
            let req_line = RequestLine::parse(&mut bytes)?;
            let (headers, body) = Self::parse_headers_and_body(&mut bytes)?;
            let request = SipRequest::new(req_line, headers, body);

            SipMessage::Request(request)
        } else {
            let status_line = StatusLine::parse(&mut bytes)?;
            let (headers, body) = Self::parse_headers_and_body(&mut bytes)?;
            let response = SipResponse::new(status_line, headers, body);

            SipMessage::Response(response)
        };

        Ok(msg)
    }

    fn is_sip_version(bytes: &Bytes) -> bool {
        const SIP: &[u8] = b"SIP";
        let tag = peek_while!(bytes, is_alphabetic);
        let next = bytes.src.get(tag.len());

        next.is_some_and(|next| tag == SIP && next == &b'/')
    }

    pub fn parse_sip_v2(bytes: &mut Bytes) -> Result<()> {
        if let Some(SIPV2) = bytes.peek_n(7) {
            bytes.nth(6);
            return Ok(());
        }
        sip_parse_error!("Sip Version Invalid")
    }

    fn parse_headers_and_body<'a>(
        bytes: &mut Bytes<'a>,
    ) -> Result<(Headers<'a>, Option<&'a [u8]>)> {
        let mut headers = Headers::new();
        let body = headers.parses_and_return_body(bytes)?;

        Ok((headers, body))
    }
}

/// Error on parsing
#[derive(Debug)]
pub struct SipParserError {
    /// Message in error
    pub message: String,
}

#[allow(missing_docs)]
impl SipParserError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for SipParserError {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for SipParserError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl From<ParseIntError> for SipParserError {
    fn from(value: ParseIntError) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl From<ParseFloatError> for SipParserError {
    fn from(value: ParseFloatError) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl<'a> From<BytesError<'a>> for SipParserError {
    fn from(err: BytesError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{} column:{} kind:{:?}
                {}",
                err.line,
                err.col,
                err.kind,
                String::from_utf8_lossy(err.src)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        message::{SipMethod, SipStatusCode},
        uri::{HostPort, Scheme},
    };

    use super::*;

    #[test]
    fn test_parses_and_return_body() {
        let headers = b"Max-Forwards: 70\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n";
        let mut bytes = Bytes::new(headers);
        let mut sip_headers = Headers::new();
        let body = sip_headers.parses_and_return_body(&mut bytes);
        assert_eq!(body.unwrap(), None);

        assert_eq!(sip_headers.len(), 5);
    }

    #[test]
    fn test_parse_req_line() {
        let req_line = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n";
        let mut bytes = Bytes::new(req_line);
        let parsed = RequestLine::parse(&mut bytes);
        let RequestLine { method, uri } = parsed.unwrap();

        assert_eq!(method, SipMethod::Register);
        assert_eq!(uri.scheme, Scheme::Sip);
        assert_eq!(
            uri.host,
            HostPort::DomainName {
                host: "registrar.biloxi.com",
                port: None
            }
        );
    }

    #[test]
    fn test_parse_sip_version_2_0() {
        let src = b"SIP/2.0";
        let mut bytes = Bytes::new(src);
        assert!(SipParser::parse_sip_v2(&mut bytes).is_ok());
    }

    #[test]
    fn status_line() {
        let msg = b"SIP/2.0 200 OK\r\n";
        let mut bytes = Bytes::new(msg);
        let parsed = StatusLine::parse(&mut bytes);
        let StatusLine {
            status_code,
            reason_phrase,
        } = parsed.unwrap();

        assert_eq!(status_code, SipStatusCode::Ok);
        assert_eq!(reason_phrase, SipStatusCode::Ok.reason_phrase());
    }
}
