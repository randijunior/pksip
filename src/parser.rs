use crate::{
    macros::{alpha, digits, newline, next, sip_parse_error, space},
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    reader::{InputReader, ReaderError},
    uri::Scheme,
};

use std::str::{self};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
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

pub fn parse_status_line<'a>(
    reader: &'a InputReader,
) -> Result<StatusLine<'a>, SipParserError> {
    reader.tag(SIPV2)?;

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let status_code = SipStatusCode::from(digits);
    let bytes = newline!(reader);

    if let Ok(rp) = str::from_utf8(bytes) {
        Ok(StatusLine::new(status_code, rp))
    } else {
        sip_parse_error!("Reason phrase is invalid utf8!")
    }
}

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

    todo!()
}

#[cfg(test)]
mod tests {
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
}
