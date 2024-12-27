use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::parser;
use crate::{macros::hdr_list, parser::Result};

use crate::headers::SipHeader;

/// The `Unsupported` SIP header.
///
/// Lists the features not supported by the `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Unsupported<'a> {
    const NAME: &'static str = "Unsupported";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = hdr_list!(reader => parser::parse_token(reader)?);

        Ok(Unsupported(tags))
    }
}

impl fmt::Display for Unsupported<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}
