//! SIP Parser

use error::SipParserError;
use scanner::peek_while;
use scanner::util::is_alphabetic;
use scanner::Scanner;

pub mod error;

use crate::macros::sip_parse_error;
use crate::uri::SIP;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

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
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        SipParserContext {
            scanner: Scanner::new(bytes),
            should_parse_body: false,
            state: ParserState::default(),
        }
    }
}

pub struct SipParser;

impl SipParser {
    /// Parse a buff of bytes into sip message
    pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
        let mut ctx = SipParserContext::from_bytes(buff);

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

#[cfg(test)]
mod tests {
    use crate::{
        headers::{self, Header},
        message::{SipMethod, SipStatusCode},
        uri::{HostPort, Scheme, Uri},
    };

    use super::*;

    macro_rules! hdr {
        ($name:ident, $bytes:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let mut ctx = SipParserContext::from_bytes($bytes);
                let mut sip_headers = Headers::new();

                assert!(sip_headers.parse(&mut ctx).is_ok());
                assert_eq!(sip_headers.len(), $expected.len());

                $expected
                    .iter()
                    .zip(sip_headers.iter())
                    .for_each(|(a, b)| assert_eq!(*a, *b));
            }
        };
    }

    macro_rules! req_line {
        ($name:ident, $bytes:expr, $method:expr, $uri:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($bytes);
                let parsed = RequestLine::parse(&mut scanner);
                let RequestLine { method, uri } = parsed.unwrap();

                assert_eq!(method, $method);
                assert_eq!(uri, $uri);
            }
        };
    }

    macro_rules! st_line {
        ($name:ident, $bytes:expr, $code:expr, $phrase:expr) => {
            #[test]
            fn $name() {
                let mut scanner = Scanner::new($bytes);
                let parsed = StatusLine::parse(&mut scanner);
                let StatusLine {
                    status_code,
                    reason_phrase,
                } = parsed.unwrap();

                assert_eq!(status_code, $code);
                assert_eq!(reason_phrase, $phrase);
            }
        };
    }

    hdr! {
        test_hdr_1,
        b"Max-Forwards: 70\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n",
        [
            Header::MaxForwards(headers::MaxForwards::new(70)),
            Header::CallId(headers::CallId::new("843817637684230@998sdasdh09")),
            Header::CSeq(headers::CSeq::new(1826, SipMethod::Register)),
            Header::Expires(headers::Expires::new(7200)),
            Header::ContentLength(headers::ContentLength::new(0)),
        ]

    }

    hdr! {
        test_hdr_2,
        b"X-Custom-Header: 12345\r\n",
        [
            Header::Other { name: "X-Custom-Header", value: "12345" }
        ]

    }

    req_line! {
            test_req_line_1,
            b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n",
            SipMethod::Register,
            Uri {
                scheme: Scheme::Sip,
                host: HostPort::DomainName {
                    host: "registrar.biloxi.com",
                    port: None
            },
            user: None,
            header_params: None,
            params: None,
            other_params: None
        }
    }

    st_line! {
        test_st_line1,
        b"SIP/2.0 200 OK\r\n",
        SipStatusCode::Ok,
        SipStatusCode::Ok.reason_phrase()
    }
}
