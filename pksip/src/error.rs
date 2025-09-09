use std::str::{
    Utf8Error, {self},
};

use thiserror::Error;
use tokio::task::JoinError;
use util::{Position, ScannerError};

use crate::message::SipMethod;

pub type Result<T> = std::result::Result<T, Error>;

impl std::convert::From<tokio::sync::mpsc::error::SendError<crate::transport::TransportMessage>>
    for Error
{
    fn from(
        value: tokio::sync::mpsc::error::SendError<crate::transport::TransportMessage>,
    ) -> Self {
        Self::ChannelClosed
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

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] ParseError),

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

    #[error("Failed to execute tokio task")]
    JoinError(JoinError),

    #[error("Internal error: {0}")]
    Internal(&'static str),

    #[error("SipMethod {0} cannot establish a dialog")]
    DialogError(#[from] DialogError),
}

#[derive(Debug, Error)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub position: Position,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, position: Position) -> Self {
        Self { kind, position }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseErrorKind {
    StatusCode,
    Header,
    Host,
    Method,
    Version,
    Uri,
    Param,
    Scanner(ScannerError),
}

#[derive(Debug, Error)]
pub enum DialogError {
    #[error("SipMethod {0} cannot establish a dialog")]
    InvalidMethod(SipMethod),

    #[error("Missing To tag in 'To' header")]
    MissingTagInToHeader,
}
