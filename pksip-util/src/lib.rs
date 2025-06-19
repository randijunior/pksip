use std::str;

pub mod macros;
pub mod util;

use crate::util::is_digit;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Position {
    /// Current line.
    line: usize,
    /// Current column.
    col: usize,
}

impl Position {
    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }
}

/// Reading byte slice while keep the line and column.
#[derive(Debug)]
pub struct Scanner<'a> {
    /// The input bytes slice to be read.
    src: &'a [u8],
    /// Current position
    pos: Position,
    /// Current index.
    idx: usize,

    len: usize,
}

impl<'a> Scanner<'a> {
    /// Create a `Scanner` from a byte slice.
    ///
    /// The `line` and `col` will always start from 1.
    pub const fn new(src: &'a [u8]) -> Self {
        Scanner {
            src,
            pos: Position { line: 1, col: 1 },
            idx: 0,
            len: src.len(),
        }
    }

    pub fn position(&self) -> &Position {
        &self.pos
    }

    /// Returns `true` if all bytes where read
    #[inline(always)]
    pub fn is_eof(&self) -> bool {
        self.idx >= self.src.len()
    }

    /// Get next byte without advance
    #[inline]
    pub fn peek(&self) -> Option<&u8> {
        self.src.get(self.idx)
    }

    /// Moves to the next character n times
    pub fn bump_n(&mut self, n: usize) {
        for _ in 0..n {
            self.next();
        }
    }

    /// Same as [Scanner::peek] but will return an `Result`
    /// instead a `Option`.
    #[inline]
    pub fn lookahead(&self) -> Result<&u8> {
        self.peek().ok_or_else(|| self.error::<u8>(ErrorKind::Eof).unwrap_err())
    }

    #[inline]
    pub fn starts_with(&self, pat: &[u8]) -> bool {
        self.src.get(self.idx..).is_some_and(|rem| rem.starts_with(pat))
    }

    /// Get `n` bytes without advance.
    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        let rem = self.as_ref();
        if rem.len() > n {
            Some(&rem[..n])
        } else {
            None
        }
    }

    /// Read a `u32` number from the slice.
    ///
    /// This method reads until an invalid digit is found.
    pub fn read_u32(&mut self) -> Result<u32> {
        let digits = unsafe { str::from_utf8_unchecked(digits!(self)) };

        match digits.parse() {
            Ok(num) => Ok(num),
            Err(_) => self.error(ErrorKind::Num),
        }
    }

    /// Read a `u16` number from the slice.
    ///
    /// This method reads until an invalid digit is found.
    pub fn read_u16(&mut self) -> Result<u16> {
        let digits = unsafe { str::from_utf8_unchecked(digits!(self)) };

        match digits.parse() {
            Ok(num) => Ok(num),
            Err(_) => self.error(ErrorKind::Num),
        }
    }

    /// `read_while()` will call the `func` closure for
    /// each element in the slice and advance
    /// while the closure returns `true`.
    ///
    /// # Returns
    ///
    /// A slice of bytes from the starting position to the position
    /// where the closure `func` returns `false` or the end of the slice
    /// is reached.
    #[inline(always)]
    pub fn read_while<F>(&mut self, func: F) -> &'a [u8]
    where
        F: Fn(u8) -> bool,
    {
        let start = self.idx;
        let src = self.src;
        let len = src.len();

        while self.idx < len && func(src[self.idx]) {
            self.bump(src[self.idx]);
        }

        &src[start..self.idx]
    }

    pub fn peek_while<F>(&self, func: F) -> (&'a [u8], Option<u8>)
    where
        F: Fn(u8) -> bool,
    {
        let start = self.idx;
        let src = &self.src[start..];

        let n = src.iter().position(|&b| !func(b)).unwrap_or(src.len());
        let next_byte = src.get(n).copied();

        (&src[..n], next_byte)
    }

    /// Checks whether the current characters match the specified slice.
    pub fn matches_slice(&mut self, slice: &[u8]) -> Result<()> {
        let start_index = self.idx;
        let slice_len = slice.len();

        let position = self
            .zip(slice.iter())
            .position(|(expected, &current)| expected != current);

        match position {
            // Invalid.
            Some(_) => self.error(ErrorKind::Tag),
            None if self.idx - start_index >= slice_len => Ok(()),
            // Incomplete.
            None => self.error(ErrorKind::Tag),
        }
    }

    /// Read next byte if equals to `b`.
    ///
    /// # Errors
    ///
    /// This method will return an error if the byte is not
    /// equal to `b`.
    ///
    /// If the slice reached the end, then an error will
    /// also be returned.
    pub fn must_read(&mut self, b: u8) -> Result<()> {
        let Some(&n) = self.peek() else {
            return self.error(ErrorKind::Eof);
        };
        if b != n {
            return self.error(ErrorKind::Char { expected: b, found: n });
        }
        self.next();
        Ok(())
    }

    pub fn take_until(&mut self, byte: u8) -> &'a [u8] {
        self.read_while(|b| b != byte)
    }

    /// Same as [Scanner::read_while] but will return the
    /// slice of bytes converted to a string slice.
    ///
    /// # Safety
    ///
    /// Caller must ensures that `func` valid that bytes are
    /// valid UTF-8.
    #[inline]
    pub unsafe fn read_as_str<F>(&mut self, func: F) -> &'a str
    where
        F: Fn(u8) -> bool,
    {
        let bytes = self.read_while(&func);

        // SAFETY: the caller must guarantee that the `func` valid
        // that bytes are valid UTF-8.
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    /// Read number in the slice.
    ///
    /// This method read until an invalid digit is found.
    pub fn read_num<N>(&mut self) -> Result<N>
    where
        N: lexical_core::FromLexical,
    {
        match lexical_core::parse_partial::<N>(self.as_ref()) {
            Ok((value, readed)) if readed > 0 => {
                self.nth(readed - 1);
                Ok(value)
            }
            _ => self.error(ErrorKind::Num),
        }
    }

    pub fn scan_number_str(&mut self) -> &'a str {
        let start = self.idx;
        self.read_while(is_digit);

        loop {
            match self.consume_if(|b| b == b'.') {
                Some(_) => continue,
                None => {
                    self.read_while(is_digit);
                    break;
                }
            }
        }
        let end = self.idx;

        unsafe { str::from_utf8_unchecked(&self.src[start..end]) }
    }

    /// Call the `func` closure for next byte and read it if
    /// the closure returns `true`.
    ///
    /// # Returns
    ///
    /// The byte readed.
    #[inline(always)]
    pub fn consume_if<F>(&mut self, func: F) -> Option<u8>
    where
        F: FnOnce(u8) -> bool,
    {
        match self.peek() {
            Some(&matched) if func(matched) => {
                self.bump(matched);
                Some(matched)
            }
            _ => None,
        }
    }

    #[inline(always)]
    fn bump(&mut self, byte: u8) {
        if byte == b'\n' {
            self.pos.col = 1;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
        self.idx += 1;
    }

    pub fn cur_is_some_and<F>(&self, func: F) -> bool
    where
        F: FnOnce(u8) -> bool,
    {
        self.peek().is_some_and(|&b| func(b))
    }

    #[inline]
    pub fn remaing(&self) -> &[u8] {
        self.as_ref()
    }

    fn error<T>(&self, kind: ErrorKind) -> Result<T> {
        Err(Error {
            kind,
            line: self.pos.line,
            col: self.pos.col,
        })
    }
}

impl ToString for Scanner<'_> {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(self.remaing()).into()
    }
}

/// Errors that can occur while reading the src.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// End of file reached.
    Eof,
    Char {
        expected: u8,
        found: u8,
    },
    Num,
    Tag,
}

#[derive(Debug, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
}

impl AsRef<[u8]> for Scanner<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        unsafe { self.src.get_unchecked(self.idx..self.len) }
    }
}

impl Iterator for Scanner<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.src.get(self.idx).copied().inspect(|&byte| self.bump(byte))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_num() {
        let mut reader = Scanner::new("12345".as_bytes());
        assert_eq!(reader.read_num(), Ok(12345));

        let mut reader = Scanner::new("NaN".as_bytes());
        assert!(reader.read_num::<u32>().is_err());
        assert_eq!(reader.as_ref(), b"NaN");

        let mut reader = Scanner::new("9123Test".as_bytes());
        assert_eq!(reader.read_num(), Ok(9123));
        assert_eq!(reader.as_ref(), b"Test");
    }

    #[test]
    fn test_lookahead() {
        let mut reader = Scanner::new("Hello".as_bytes());

        assert_eq!(reader.lookahead(), Ok(&b'H'));
        reader.next();
        assert_eq!(reader.lookahead(), Ok(&b'e'));
        reader.next();
        assert_eq!(reader.lookahead(), Ok(&b'l'));

        reader.read_while(|_| true);

        assert!(reader.lookahead().is_err());
    }
}
