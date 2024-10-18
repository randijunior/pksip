/*#![deny(
    missing_docs,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks
)]
*/

//! # sip-rs
//!
//! Library for parse SIP message
//! 
//! ## Example
//! ```rust


pub mod headers;
pub mod msg;
pub mod parser;
pub mod uri;


pub(crate) mod macros;
pub(crate) mod scanner;
pub(crate) mod util;

#[cfg(test)]
#[macro_use]
extern crate assert_matches;
