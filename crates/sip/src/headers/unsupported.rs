use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::internal::ArcStr;
use crate::parser;
use crate::{macros::hdr_list, parser::Result};

use crate::headers::SipHeader;

/// The `Unsupported` SIP header.
///
/// Lists the features not supported by the `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Unsupported(Vec<ArcStr>);

impl SipHeader<'_> for Unsupported {
    const NAME: &'static str = "Unsupported";
    /*
     * Unsupported  =  "Unsupported" HCOLON option-tag *(COMMA option-tag)
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let tags = hdr_list!(reader => parser::parse_token(reader)?.into());

        Ok(Unsupported(tags))
    }
}

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}
