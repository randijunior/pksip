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

#[inline(always)]
pub(crate) fn alphanum(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

#[inline(always)]
fn mark(byte: u8) -> bool {
    match byte {
        b'-' | b'_' | b'.' | b'!' | b'~' | b'*' | b'\'' | b'(' | b')' => {
            true
        }
        _ => false,
    }
}

#[inline(always)]
pub(crate) fn uneserved(byte: u8) -> bool {
    alphanum(byte) || mark(byte)
}
#[inline(always)]
pub(crate) fn escaped(byte: u8) -> bool {
    byte == b'%'
}
