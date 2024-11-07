#[inline(always)]
pub fn is_digit(c: &u8) -> bool {
    c.is_ascii_digit()
}

#[inline(always)]
pub fn maybe_a_number(c: &u8) -> bool {
    is_digit(c) || c == &b'.'
}

#[inline(always)]
pub fn is_space(c: &u8) -> bool {
    c == &b' ' || c == &b'\t'
}

#[inline(always)]
pub fn is_newline(c: &u8) -> bool {
    c == &b'\r' || c == &b'\n'
}

#[inline(always)]
pub fn not_comma_or_newline(c: &u8) -> bool {
    !matches!(c, b',' | b'\r' | b'\n')
}
#[inline(always)]
pub fn is_alphabetic(c: &u8) -> bool {
    c.is_ascii_alphabetic()
}

#[inline(always)]
pub fn is_valid_port(v: u16) -> bool {
    matches!(v, 0..=65535)
}
