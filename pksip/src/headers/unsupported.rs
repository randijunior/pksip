use std::{fmt, str};

use itertools::Itertools;

use crate::parser::Parser;
use crate::{error::Result, macros::hdr_list};

use crate::headers::SipHeaderParse;

/// The `Unsupported` SIP header.
///
/// Lists the features not supported by the `UAS`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParse<'a> for Unsupported<'a> {
    const NAME: &'static str = "Unsupported";
    /*
     * Unsupported  =  "Unsupported" HCOLON option-tag *(COMMA option-tag)
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let tags = hdr_list!(parser => parser.parse_token()?);

        Ok(Unsupported(tags))
    }
}

impl fmt::Display for Unsupported<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", Unsupported::NAME, self.0.iter().format(", "))
    }
}
