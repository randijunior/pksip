use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::internal::ArcStr;
use crate::macros::hdr_list;
use crate::parser::{self, Result};

use crate::headers::SipHeader;

/// The `Require` SIP header.
///
/// Is used by `UACs` to tell `UASs` about options that the
/// `UAC` expects the `UAS` to support in order to process the
/// request.
#[derive(Debug, PartialEq, Eq)]
pub struct Require(Vec<ArcStr>);

impl SipHeader<'_> for Require {
    const NAME: &'static str = "Require";
    /*
     * Require  =  "Require" HCOLON option-tag *(COMMA option-tag)
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let tags =
            hdr_list!(reader => parser::parse_token(reader)?.into());

        Ok(Require(tags))
    }
}

impl fmt::Display for Require {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel\r\n";
        let mut reader = Reader::new(src);
        let require = Require::parse(&mut reader);
        let require = require.unwrap();

        assert_eq!(require.0.get(0), Some(&"100rel".into()));
    }
}
