use std::str;

use crate::{
    bytes::Bytes,
    macros::{b_map, until_byte},
    parser::{Result, ALPHA_NUM, TOKEN},
};

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

pub struct Token;

impl<'a> Token {
    #[inline]
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> &'a str {
        // is_token ensures that is valid UTF-8
        unsafe { bytes.read_and_convert_to_str(is_token) }
    }

    pub fn parse_quoted(bytes: &mut Bytes<'a>) -> Result<&'a str> {
        if let Some(&b'"') = bytes.peek() {
            bytes.next();
            let value = until_byte!(bytes, &b'"');
            bytes.next();

            Ok(str::from_utf8(value)?)
        } else {
            Ok(Self::parse(bytes))
        }
    }
}

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC_MAP[*b as usize]
}
