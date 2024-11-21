use std::str;

use scanner::{until_byte, Scanner};

use crate::{
    macros::b_map,
    parser::{Result, ALPHA_NUM, TOKEN},
};

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

pub struct Token;

impl<'a> Token {
    #[inline]
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> &'a str {
        // is_token ensures that is valid UTF-8
        unsafe { scanner.read_and_convert_to_str_while(Self::is_token) }
    }

    #[inline(always)]
    pub(crate) fn is_token(b: &u8) -> bool {
        TOKEN_SPEC_MAP[*b as usize]
    }

    pub fn parse_quoted(scanner: &mut Scanner<'a>) -> Result<&'a str> {
        if let Some(&b'"') = scanner.peek() {
            scanner.next();
            let value = until_byte!(scanner, &b'"');
            scanner.next();

            Ok(str::from_utf8(value)?)
        } else {
            Ok(Self::parse(scanner))
        }
    }
}
