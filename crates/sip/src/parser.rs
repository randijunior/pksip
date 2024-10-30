//! SIP Parser

use crate::bytes::Bytes;
use crate::macros::sip_parse_error;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::str::Utf8Error;

use crate::bytes::BytesError;
use crate::message::headers::Headers;

use crate::macros::b_map;
use crate::macros::peek_while;
use crate::macros::read_while;

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

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC_MAP[*b as usize]
}

#[inline]
pub(crate) fn parse_slice_utf8<'a>(
    bytes: &mut Bytes<'a>,
    func: impl Fn(&u8) -> bool,
) -> &'a str {
    let slc = read_while!(bytes, func);

    // SAFETY: caller must ensures that func valid that bytes are valid UTF-8
    unsafe { str::from_utf8_unchecked(slc) }
}

#[inline]
pub(crate) fn parse_token<'a>(bytes: &mut Bytes<'a>) -> &'a str {
    // is_token ensures that is valid UTF-8
    parse_slice_utf8(bytes, is_token)
}

pub(crate) fn parse_sip_v2(bytes: &mut Bytes) -> Result<()> {
    if let Some(SIPV2) = bytes.peek_n(7) {
        bytes.nth(6);
        return Ok(());
    }
    sip_parse_error!("Sip Version Invalid")
}

fn is_sip_version(bytes: &Bytes) -> bool {
    const SIP: &[u8] = b"SIP";
    let tag = peek_while!(bytes, is_alphabetic);
    let next = bytes.src.get(tag.len());

    next.is_some_and(|next| tag == SIP && next == &b'/')
}

fn parse_headers_and_body<'a>(
    bytes: &mut Bytes<'a>,
) -> Result<(Headers<'a>, Option<&'a [u8]>)> {
    let mut headers = Headers::new();
    let body = headers.parse_headers_and_return_body(bytes)?;

    Ok((headers, body))
}

/// Parse a buff of bytes into sip message
pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
    let mut bytes = Bytes::new(buff);

    let msg = if !is_sip_version(&bytes) {
        let req_line = RequestLine::parse(&mut bytes)?;
        let (headers, body) = parse_headers_and_body(&mut bytes)?;
        let request = SipRequest::new(req_line, headers, body);

        SipMessage::Request(request)
    } else {
        let status_line = StatusLine::parse(&mut bytes)?;
        let (headers, body) = parse_headers_and_body(&mut bytes)?;
        let response = SipResponse::new(status_line, headers, body);

        SipMessage::Response(response)
    };

    Ok(msg)
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
        message::headers::{
            call_id::CallId, content_length::ContentLength, cseq::CSeq,
            expires::Expires, max_fowards::MaxForwards, Header,
        },
        message::{SipMethod, SipStatusCode},
        uri::{HostPort, Scheme},
    };

    use super::*;

    #[test]
    fn test_parse_headers_and_return_body() {
        let headers = b"Max-Forwards: 70\r\n\
        Call-ID: 843817637684230@998sdasdh09\r\n\
        CSeq: 1826 REGISTER\r\n\
        Expires: 7200\r\n\
        Content-Length: 0\r\n\r\n";
        let mut bytes = Bytes::new(headers);
        let mut sip_headers = Headers::new();
        let body = sip_headers.parse_headers_and_return_body(&mut bytes);
        assert_eq!(body.unwrap(), None);

        assert_eq!(sip_headers.len(), 5);
    }

    #[test]
    fn test_parse_req_line() {
        let req_line = b"REGISTER sip:registrar.biloxi.com SIP/2.0\r\n";
        let mut bytes = Bytes::new(req_line);
        let parsed = RequestLine::parse(&mut bytes);
        let parsed = parsed.unwrap();

        match parsed {
            RequestLine { method, uri } => {
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
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_parse_sip_version_2_0() {
        let src = b"SIP/2.0";
        let mut bytes = Bytes::new(src);
        parse_sip_v2(&mut bytes).unwrap();
    }

    #[test]
    fn status_line() {
        let msg = b"SIP/2.0 200 OK\r\n";
        let mut bytes = Bytes::new(msg);
        let parsed = StatusLine::parse(&mut bytes);
        let parsed = parsed.unwrap();

        match parsed {
            StatusLine {
                status_code,
                reason_phrase,
            } => {
                assert_eq!(status_code, SipStatusCode::Ok);
                assert_eq!(reason_phrase, SipStatusCode::Ok.reason_phrase());
            }
            _ => unreachable!(),
        }
    }
}
