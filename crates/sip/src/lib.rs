//! SIP types and parser

pub mod message;
pub mod parser;
pub mod uri;
pub mod headers;


pub(crate) mod bytes;
pub(crate) mod macros;
pub(crate) mod token;
pub(crate) mod util;
pub(crate) mod params;
pub(crate) mod auth;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
