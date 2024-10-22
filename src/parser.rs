use crate::scanner::Scanner;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::str::Utf8Error;

use crate::headers::Headers;
use crate::scanner::ScannerError;

use crate::macros::b_map;
use crate::macros::peek_while;
use crate::macros::read_while;

use crate::msg::SipMsg;
use crate::msg::StatusLine;
use crate::msg::{RequestLine, SipRequest, SipResponse};

use crate::util::is_alphabetic;
use crate::util::is_space;

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

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC_MAP[*b as usize]
}

#[inline]
pub(crate) fn parse_slice_utf8<'a>(
    scanner: &mut Scanner<'a>,
    func: impl Fn(&u8) -> bool,
) -> &'a str {
    let slc = read_while!(scanner, func);

    // SAFETY: caller must ensures that func valid that bytes are valid utf-8
    unsafe { str::from_utf8_unchecked(slc) }
}

#[inline]
pub(crate) fn parse_token<'a>(scanner: &mut Scanner<'a>) -> &'a str {
    // is_token ensures that is valid utf-8
    parse_slice_utf8(scanner, is_token)
}

fn is_sip_version(scanner: &Scanner) -> bool {
    const SIP: &[u8] = b"SIP";
    let tag = peek_while!(scanner, is_alphabetic);
    let next = scanner.src.get(tag.len());

    next.is_some_and(|next| tag == SIP && (next == &b'/' || is_space(next)))
}

fn parse_headers_and_body<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<(Headers<'a>, Option<&'a [u8]>)> {
    let mut headers = Headers::new();
    let body = headers.parse_headers(scanner)?;

    Ok((headers, body))
}

/// Parse a buff of bytes into sip message
pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMsg<'a>> {
    let mut scanner = Scanner::new(buff);

    let msg = if !is_sip_version(&scanner) {
        let req_line = RequestLine::parse(&mut scanner)?;
        let (headers, body) = parse_headers_and_body(&mut scanner)?;
        let request = SipRequest::new(req_line, headers, body);

        SipMsg::Request(request)
    } else {
        let status_line = StatusLine::parse(&mut scanner)?;
        let (headers, body) = parse_headers_and_body(&mut scanner)?;
        let response = SipResponse::new(status_line, headers, body);

        SipMsg::Response(response)
    };

    Ok(msg)
}

/// Error on parsing
#[derive(Debug, PartialEq)]
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
        headers::{
            common::{call_id::CallId, cseq::CSeq, max_fowards::MaxForwards},
            control::expires::Expires,
            session::content_length::ContentLength,
            Header,
        },
        msg::{SipMethod, SipStatusCode},
        uri::{HostPort, Scheme},
    };

    use super::*;

    #[test]
    fn test_parse_headers() {
        let headers = b"Max-Forwards: 70\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n";
        let mut scanner = Scanner::new(headers);
        let mut sip_headers = Headers::new();
        let body = sip_headers.parse_headers(&mut scanner);
        assert_eq!(body.unwrap(), None);

        let mut iter = sip_headers.iter();
        assert_eq!(
            iter.next().unwrap(),
            &Header::MaxForwards(MaxForwards::new(70))
        );
        assert_eq!(
            iter.next().unwrap(),
            &Header::CallId(CallId::new("843817637684230@998sdasdh09"))
        );
        assert_eq!(
            iter.next().unwrap(),
            &Header::CSeq(CSeq::new(1826, SipMethod::Register))
        );
        assert_eq!(iter.next().unwrap(), &Header::Expires(Expires::new(7200)));
        assert_eq!(
            iter.next().unwrap(),
            &Header::ContentLength(ContentLength::new(0))
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_parse_req_line() {
        let req_line = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n";
        let mut scanner = Scanner::new(req_line);
        let parsed = RequestLine::parse(&mut scanner);
        let parsed = parsed.unwrap();

        assert_matches!(parsed, RequestLine { method, uri } => {
            assert_eq!(method, SipMethod::Register);
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(uri.user, None);
            assert_eq!(uri.host, HostPort::DomainName { host: "registrar.biloxi.com", port: None });
        });
    }

    #[test]
    fn status_line() {
        let msg = b"SIP/2.0 200 OK\r\n";
        let mut scanner = Scanner::new(msg);
        let parsed = StatusLine::parse(&mut scanner);
        let parsed = parsed.unwrap();

        assert_matches!(parsed, StatusLine { status_code, reason_phrase } => {
            assert_eq!(status_code, SipStatusCode::Ok);
            assert_eq!(reason_phrase, SipStatusCode::Ok.reason_phrase());
        });
    }
}
