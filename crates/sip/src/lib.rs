pub mod auth;
pub mod endpoint;
pub mod headers;
pub mod internal;
pub mod message;
pub mod parser;
pub mod service;
pub mod transaction;
pub mod transport;

pub(crate) mod macros;
pub(crate) mod resolver;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
