//! SIP types and parser

pub mod headers;
pub mod msg;
pub mod parser;
pub mod transport;
pub mod endpoint;

pub(crate) mod auth;
pub(crate) mod macros;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
