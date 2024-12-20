//! SIP types and parser

pub mod headers;
pub mod msg;
pub mod parser;
pub mod server;
pub mod transport;

pub(crate) mod auth;
pub(crate) mod macros;
pub(crate) mod resolver;
pub(crate) mod serializer;
pub(crate) mod service;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
