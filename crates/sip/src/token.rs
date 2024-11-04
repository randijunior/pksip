use core::str;

use crate::{
    bytes::Bytes,
    macros::{b_map, read_until_byte, read_while, sip_parse_error},
    parser::{Result, ALPHA_NUM, TOKEN},
};

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

pub struct Token;

impl<'a> Token {
    #[inline]
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> &'a str {
        // is_token ensures that is valid UTF-8
        Self::parse_slice(bytes, is_token)
    }

    #[inline]
    pub(crate) fn parse_slice<F>(bytes: &mut Bytes<'a>, func: F) -> &'a str
    where
        F: Fn(&u8) -> bool,
    {
        let slc = read_while!(bytes, func);

        // SAFETY: caller must ensures that func valid that bytes are valid UTF-8
        unsafe { str::from_utf8_unchecked(slc) }
    }

    pub fn parse_quoted(bytes: &mut Bytes<'a>) -> Result<&'a str> {
        if let Some(&b'"') = bytes.peek() {
            bytes.next();
            let value = read_until_byte!(bytes, &b'"');
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
