use core::str;

use crate::{
    bytes::Bytes, macros::parse_header_list, parser::Result, token::Token,
};

use crate::headers::SipHeader;

/// Lists the features not supported by the `UAS`.
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for Unsupported<'a> {
    const NAME: &'static str = "Unsupported";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tags = parse_header_list!(bytes => Token::parse(bytes));

        Ok(Unsupported(tags))
    }
}
