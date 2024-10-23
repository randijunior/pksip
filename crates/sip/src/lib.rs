//! SIP types and parser



pub mod message;
pub mod parser;
pub mod uri;

pub(crate) mod macros;
pub(crate) mod scanner;
pub(crate) mod util;

pub use message::headers;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
