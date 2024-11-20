//! SIP types and parser

pub mod headers;
pub mod message;
pub mod parser;
pub mod uri;

pub(crate) mod auth;
pub(crate) mod macros;
pub(crate) mod params;
pub(crate) mod token;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
