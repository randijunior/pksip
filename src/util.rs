#[inline(always)]
pub fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[inline(always)]
pub fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

#[inline(always)]
pub fn is_newline(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

#[inline(always)]
pub fn is_alphabetic(c: u8) -> bool {
    c.is_ascii_alphabetic()
}
