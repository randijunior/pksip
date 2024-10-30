use core::str;

use crate::{
    bytes::Bytes,
    macros::{read_while, space},
    parser::{self, is_token, Result},
};

use crate::headers::SipHeaderParser;

/// Lists the features not supported by the `UAS`.
pub struct Unsupported<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for Unsupported<'a> {
    const NAME: &'static str = "Unsupported";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let tag = parser::parse_token(bytes);
        let mut tags = vec![tag];

        while let Some(b',') = bytes.peek() {
            let tag = read_while!(bytes, is_token);
            let tag = unsafe { str::from_utf8_unchecked(tag) };
            tags.push(tag);
            space!(bytes);
        }

        Ok(Unsupported(tags))
    }
}
