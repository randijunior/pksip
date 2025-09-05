use std::sync::Arc;
use std::{fmt, str};

use itertools::Itertools;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::macros::comma_separated_header_value;
use crate::parser::Parser;

/// The `Unsupported` SIP header.
///
/// Lists the features not supported by the `UAS`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unsupported(Vec<Arc<str>>);

impl<'a> HeaderParser<'a> for Unsupported {
    const NAME: &'static str = "Unsupported";

    /*
     * Unsupported  =  "Unsupported" HCOLON option-tag
     * *(COMMA option-tag)
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let tags = comma_separated_header_value!(parser => parser.parse_token()?.into());

        Ok(Unsupported(tags))
    }
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Unsupported::NAME, self.0.iter().format(", "))
    }
}
