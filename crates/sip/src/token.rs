use std::str;

use reader::{until_byte, Reader};

use crate::{
    macros::b_map,
    parser::{Result, ALPHA_NUM, TOKEN},
};

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

pub struct Token;

impl<'a> Token {
    #[inline]
    pub(crate) fn parse(reader: &mut Reader<'a>) -> &'a str {
        // is_token ensures that is valid UTF-8
        unsafe { reader.read_while_as_str(Self::is_token) }
    }

    #[inline(always)]
    pub(crate) fn is_token(b: &u8) -> bool {
        TOKEN_SPEC_MAP[*b as usize]
    }

    pub fn parse_quoted(reader: &mut Reader<'a>) -> Result<&'a str> {
        if let Some(&b'"') = reader.peek() {
            reader.next();
            let value = until_byte!(reader, &b'"');
            reader.next();

            Ok(str::from_utf8(value)?)
        } else {
            Ok(Self::parse(reader))
        }
    }
}
