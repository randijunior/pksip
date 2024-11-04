use core::str;

use crate::{
    bytes::Bytes,
    macros::{b_map, read_while},
    parser::{ALPHA_NUM, TOKEN},
};

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

pub struct Token;

impl Token {
    #[inline]
    pub(crate) fn parse<'a>(bytes: &mut Bytes<'a>) -> &'a str {
        // is_token ensures that is valid UTF-8
        Self::parse_slice(bytes, is_token)
    }

    #[inline]
    pub(crate) fn parse_slice<'a, F>(bytes: &mut Bytes<'a>, func: F) -> &'a str
    where
        F: Fn(&u8) -> bool,
    {
        let slc = read_while!(bytes, func);

        // SAFETY: caller must ensures that func valid that bytes are valid UTF-8
        unsafe { str::from_utf8_unchecked(slc) }
    }
}

#[inline(always)]
pub(crate) fn is_token(b: &u8) -> bool {
    TOKEN_SPEC_MAP[*b as usize]
}
