//! SIP types and parser

pub mod endpoint;
pub mod headers;
pub mod msg;
pub mod parser;
pub mod transport;

pub(crate) mod auth;
pub(crate) mod macros;
pub(crate) mod resolver;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
