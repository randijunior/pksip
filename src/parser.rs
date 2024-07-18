use reader::{InputReader, ReaderError};

use crate::{
    macros::{alpha, digits, newline, next, sip_parse_error, space},
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    uri::{Host, Scheme, Uri, UserInfo},
    util::is_newline,
};

use std::str::{self, Utf8Error};

mod cursor;
pub mod reader;

const SIPV2: &[u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    pub(crate) message: String,
}

impl<'a> From<ReaderError<'a>> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line: {}, column: {}, kind: {:?}, input: '{}'",
                err.pos.line,
                err.pos.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

impl From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: value.to_string(),
        }
    }
}

#[inline(always)]
fn maybe_has_user(reader: &InputReader) -> Option<u8> {
    for &byte in reader.as_slice().iter() {
        match byte {
            b'@' | b' ' | b'\n' | b'>' => return Some(byte),
            _ => continue,
        }
    }
    None
}

#[inline]
pub fn parse_status_line<'a>(
    reader: &'a InputReader,
) -> Result<StatusLine<'a>, SipParserError> {
    reader.tag(SIPV2)?;

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let status_code = SipStatusCode::from(digits);
    let bytes = reader.read_while(|b| !is_newline(b))?;
    let reason_phrase = str::from_utf8(bytes)?;

    newline!(reader);

    Ok(StatusLine::new(status_code, reason_phrase))
}

fn parse_uri_host<'a>(reader: &'a InputReader) -> Result<Host<'a>, SipParserError> {
    if let Some(_) = reader.next_if(|b| b == b'[')? {
        Host::parse_ipv6(reader)
    } else {
        Host::parse(reader)
    }
}

fn parse_port(reader: &InputReader) -> Result<Option<u16>, SipParserError> {
    if let Some(_) = reader.next_if(|b| b == b':')? {
        Uri::parse_port(reader)
    } else {
        Ok(None)
    }
}

#[inline]
pub fn parse_request_line<'a>(
    reader: &'a InputReader,
) -> Result<RequestLine<'a>, SipParserError> {
    let method = SipMethod::from(alpha!(reader));

    space!(reader);

    let scheme = match reader.read_until_b(b':')? {
        b"sip" => Ok(Scheme::Sip),
        b"sips" => Ok(Scheme::Sips),
        _ => sip_parse_error!("Can't parse sip uri scheme"),
    }?;

    next!(reader);

    let has_user = maybe_has_user(reader).is_some_and(|b| b == b'@');
    let user_info = if has_user {
        Some(UserInfo::parse(reader)?)
    } else {
        None
    };
    let host = parse_uri_host(reader)?;
    let port = parse_port(reader)?;
    let uri = Uri::new(scheme, user_info, host, port);

    newline!(reader);

    Ok(RequestLine { method, uri })
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_parse_status_line() {
        let sc_ok = SipStatusCode::Ok;
        let buf = "SIP/2.0 200 OK\r\n".as_bytes();
        let reader = InputReader::new(buf);

        assert_eq!(
            parse_status_line(&reader),
            Ok(StatusLine {
                status_code: sc_ok,
                reason_phrase: sc_ok.reason_phrase()
            })
        );
        let sc_not_found = SipStatusCode::NotFound;
        let buf = "SIP/2.0 404 Not Found\r\n".as_bytes();
        let reader = InputReader::new(buf);

        assert_eq!(
            parse_status_line(&reader),
            Ok(StatusLine {
                status_code: sc_not_found,
                reason_phrase: sc_not_found.reason_phrase()
            })
        );
    }

    #[test]
    fn test_req_status_line() {
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let addr: IpAddr = "10.1.1.7".parse().unwrap();
        let reader = InputReader::new(msg);
        assert_eq!(
            parse_request_line(&reader),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: Uri {
                    scheme: Scheme::Sip,
                    user: Some(UserInfo {
                        name: "1000b3",
                        password: None
                    }),
                    host: Host::IpAddr(addr),
                    port: Some(8089)
                }
            })
        );
    }
}
