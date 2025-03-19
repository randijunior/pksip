use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::internal::ArcStr;
use crate::macros::hdr_list;
use crate::parser;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Supported` SIP header.
///
/// Enumerates all the extensions supported by the `UAC` or `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Supported(Vec<ArcStr>);

impl SipHeader<'_> for Supported {
    const NAME: &'static str = "Supported";
    const SHORT_NAME: &'static str = "k";
    /*
     * Supported  =  ( "Supported" / "k" ) HCOLON
     *               [option-tag *(COMMA option-tag)]
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let tags =
            hdr_list!(reader => parser::parse_token(reader)?.into());

        Ok(Supported(tags))
    }
}

impl fmt::Display for Supported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"100rel, other\r\n";
        let mut reader = Reader::new(src);
        let supported = Supported::parse(&mut reader);
        let supported = supported.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(supported.0.get(0), Some(&"100rel".into()));
        assert_eq!(supported.0.get(1), Some(&"other".into()));
    }
}
