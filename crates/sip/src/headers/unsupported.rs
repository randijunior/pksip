use std::str;

use reader::Reader;

use crate::{
    macros::parse_header_list, parser::Result,token::Token,
};

use crate::headers::SipHeader;

/// The `Unsupported` SIP header.
///
/// Lists the features not supported by the `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Unsupported<'a> {
    const NAME: &'static str = "Unsupported";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let tags = parse_header_list!(reader => Token::parse(reader));

        Ok(Unsupported(tags))
    }
}
