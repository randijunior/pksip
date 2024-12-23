use std::{
    fmt,
    io::{self, Write},
    str,
};

use arrayvec::ArrayVec;

use crate::{
    headers::Headers,
    parser::SIPV2,
    transport::{MsgBuffer, MAX_PACKET_SIZE},
};

use super::StatusCode;

/// Represents an SIP Status-Line.
#[derive(Debug)]
pub struct StatusLine<'sl> {
    // Status Code
    pub code: StatusCode,
    // Reason String
    pub rphrase: &'sl str,
}

impl fmt::Display for StatusLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SIPV2} {} {}\r\n", self.code.as_str(), self.rphrase)
    }
}

impl<'sl> StatusLine<'sl> {
    pub fn new(st: StatusCode, rp: &'sl str) -> Self {
        StatusLine {
            code: st,
            rphrase: rp,
        }
    }
}

#[derive(Debug)]
pub struct SipResponse<'a> {
    pub st_line: StatusLine<'a>,
    pub headers: Headers<'a>,
    pub body: Option<&'a [u8]>,
}

impl<'a> SipResponse<'a> {
    pub fn new(
        st_line: StatusLine<'a>,
        headers: Headers<'a>,
        body: Option<&'a [u8]>,
    ) -> Self {
        Self {
            body,
            st_line,
            headers,
        }
    }

    pub fn encode(&self) -> io::Result<MsgBuffer> {
        let mut buf = ArrayVec::<u8, MAX_PACKET_SIZE>::new();

        write!(buf, "{}", self.st_line)?;
        write!(buf, "{}", self.headers)?;
        write!(buf, "\r\n")?;
        if let Some(body) = self.body {
            if let Err(_err) = buf.try_extend_from_slice(body) {
                return Err(io::Error::other(
                    "Packet size exceeds MAX_PACKET_SIZE",
                ));
            }
        }

        Ok(buf)
    }
}
