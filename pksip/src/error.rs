use std::str::{
    Utf8Error, {self},
};

use thiserror::Error;
use util::{Position, ScannerError};

use crate::message::SipMethod;

pub type Result<T> = std::result::Result<T, Error>;

// impl std::convert::From<tokio::sync::mpsc::error::SendError<crate::transport::TransportMessage>>
//     for Error
// {
//     fn from(
//         value:
// tokio::sync::mpsc::error::SendError<crate::transport::TransportMessage>,
//     ) -> Self {
//         Self::ChannelClosed
//     }
// }

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

    #[error("Transaction Erro: {0}")]
    TransactionError(String),

    #[error(transparent)]
    DialogError(#[from] DialogError),

    #[error("Missing required '{0}' header")]
    MissingHeader(&'static str),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Poisoned lock")]
    PoisonedLock,

    #[error("Fmt Error")]
    FmtError(std::fmt::Error),

    #[error("Internal error: {0}")]
    Internal(&'static str),
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
    Transport,
    Scanner(ScannerError),
}

#[derive(Debug, Error)]
pub enum DialogError {
    #[error("SipMethod {0} cannot establish a dialog")]
    InvalidMethod(SipMethod),

    #[error("Missing To tag in 'To' header")]
    MissingTagInToHeader,
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Invalid method for transaction creation: expected {required}, got {actual}")]
    InvalidMethod {
        required: SipMethod,
        actual: SipMethod,
    },
    #[error("OutgoingMessageInfo not present in outgoing request")]
    MissingOutgoingMessageInfo,
}
