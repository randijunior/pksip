use std::fmt;
use std::str::{self, Utf8Error};

pub type Result<T> = std::result::Result<T, Error>;

/// Error on parsing
#[derive(Debug, PartialEq, Eq, Error)]
pub struct SipParserError {
    /// Message in error
    pub message: String,
}

impl fmt::Display for SipParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[allow(missing_docs)]
impl SipParserError {
    pub fn new<T>(s: T) -> Self
    where
        T: AsRef<str>,
    {
        Self {
            message: s.as_ref().to_string(),
        }
    }
}

impl std::convert::From<&str> for SipParserError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl std::convert::From<String> for SipParserError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::convert::From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl std::convert::From<pksip_util::Error> for SipParserError {
    fn from(err: pksip_util::Error) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{} column:{} kind:{:?}",
                err.line, err.col, err.kind,
            ),
        }
    }
}

impl std::convert::From<tokio::sync::mpsc::error::SendError<crate::transport::TransportEvent>> for Error {
    fn from(value: tokio::sync::mpsc::error::SendError<crate::transport::TransportEvent>) -> Self {
        Self::ChannelClosed
    }
}

impl std::convert::From<pksip_util::Error> for Error {
    fn from(err: pksip_util::Error) -> Self {
        todo!()
    }
}

impl std::convert::From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        todo!()
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::FmtError(value)
    }
}

use thiserror::Error;

use crate::transaction::key::TsxKey;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] SipParserError),

    #[error("Missing required '{0}' header")]
    MissingRequiredHeader(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("The Sec-WebSocket-Key has missing")]
    MissingSecWebSocketKey,

    #[error("Invalid Web Socket Version")]
    InvalidWebSocketVersion,

    #[error("Fmt Error")]
    FmtError(std::fmt::Error),

    #[error("The Sec-WebSocket-Protocol is invalid")]
    InvalidSecWebSocketProtocol,
}
