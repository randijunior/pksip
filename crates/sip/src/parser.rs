//! SIP Parser

use scanner::peek_while;
use scanner::util::is_alphabetic;
use scanner::Scanner;
use scanner::ScannerError;

use crate::macros::sip_parse_error;
use crate::uri::SIP;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use std::str;
use std::str::Utf8Error;

use crate::headers::Headers;

use crate::message::SipMessage;
use crate::message::StatusLine;
use crate::message::{RequestLine, SipRequest, SipResponse};

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

#[derive(Default)]
pub enum ParserState {
    #[default]
    StartLine,
    Headers,
    Body,
}

pub struct SipParserContext<'a> {
    pub state: ParserState,
    pub should_parse_body: bool,
    pub scanner: Scanner<'a>,
}

impl<'a> SipParserContext<'a> {
    pub fn from_buff(buff: &'a [u8]) -> Self {
        SipParserContext {
            scanner: Scanner::new(buff),
            should_parse_body: false,
            state: ParserState::default(),
        }
    }
}

pub struct SipParser;

impl SipParser {
    /// Parse a buff of bytes into sip message
    pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
        let mut ctx = SipParserContext::from_buff(buff);

        if Self::is_sip_version(&ctx) {
            let status_line = StatusLine::parse(&mut ctx.scanner)?;
            let headers = Self::parse_headers(&mut ctx)?;
            let body = Self::parse_body(&mut ctx);

            return Ok(SipMessage::Response(SipResponse::new(
                status_line,
                headers,
                body,
            )));
        };

        let req_line = RequestLine::parse(&mut ctx.scanner)?;
        let headers = Self::parse_headers(&mut ctx)?;
        let body = Self::parse_body(&mut ctx);

        Ok(SipMessage::Request(SipRequest::new(
            req_line, headers, body,
        )))
    }

    fn is_sip_version(ctx: &SipParserContext) -> bool {
        let tag = peek_while!(ctx.scanner, is_alphabetic);
        let next = ctx.scanner.src.get(tag.len());

        next.is_some_and(|next| tag == SIP && next == &b'/')
    }

    pub fn parse_sip_v2(scanner: &mut Scanner) -> Result<()> {
        if let Some(SIPV2) = scanner.peek_n(7) {
            scanner.nth(6);
            return Ok(());
        }
        sip_parse_error!("Sip Version Invalid")
    }

    fn parse_headers<'a>(
        ctx: &mut SipParserContext<'a>,
    ) -> Result<Headers<'a>> {
        ctx.state = ParserState::Headers;

        let mut headers = Headers::new();
        headers.parse(ctx)?;

        Ok(headers)
    }

    fn parse_body<'a>(ctx: &mut SipParserContext<'a>) -> Option<&'a [u8]> {
        ctx.state = ParserState::Body;
        if ctx.should_parse_body {
            let idx = ctx.scanner.idx();
            Some(&ctx.scanner.src[idx..])
        } else {
            None
        }
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

impl<'a> From<ScannerError<'a>> for SipParserError {
    fn from(err: ScannerError) -> Self {
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
        let mut ctx = SipParserContext::from_buff(headers);
        let mut sip_headers = Headers::new();

        assert!(sip_headers.parse(&mut ctx).is_ok());

        assert_eq!(sip_headers.len(), 5);
    }

    #[test]
    fn test_parse_req_line() {
        let req_line = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n";
        let mut scanner = Scanner::new(req_line);
        let parsed = RequestLine::parse(&mut scanner);
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
        let mut scanner = Scanner::new(src);
        assert!(SipParser::parse_sip_v2(&mut scanner).is_ok());
    }

    #[test]
    fn status_line() {
        let msg = b"SIP/2.0 200 OK\r\n";
        let mut scanner = Scanner::new(msg);
        let parsed = StatusLine::parse(&mut scanner);
        let StatusLine {
            status_code,
            reason_phrase,
        } = parsed.unwrap();

        assert_eq!(status_code, SipStatusCode::Ok);
        assert_eq!(reason_phrase, SipStatusCode::Ok.reason_phrase());
    }
}
