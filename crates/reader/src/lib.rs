use std::ops::Range;
use std::str;

pub mod macros;
pub mod util;

use crate::util::is_digit;

type Result<'a, T> = std::result::Result<T, Error<'a>>;

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
pub struct Reader<'a> {
    /// The input bytes slice to be read.
    pub src: &'a [u8],
    /// Indicates if the reading is complete.
    finished: bool,
    /// Total length of the input slice.
    len: usize,
    /// Current position
    pub pos: Position,
    /// Current index.
    idx: usize,
}

impl<'a> Reader<'a> {
    /// Create a `Reader` from a byte slice.
    ///
    /// The `line` and `col` will always start from 1.
    pub fn new(src: &'a [u8]) -> Self {
        Reader {
            src,
            len: src.len(),
            finished: false,
            pos: Position { line: 1, col: 1 },
            idx: 0,
        }
    }

    pub fn pos(&self) -> &Position {
        &self.pos
    }

    /// Returns the current index
    #[inline]
    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Returns the length of the bytes slice
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if all bytes where read
    #[inline(always)]
    pub fn is_eof(&self) -> bool {
        self.finished
    }

    /// Get next byte without advance
    #[inline]
    pub fn peek(&self) -> Option<&u8> {
        if self.is_eof() {
            return None;
        }

        Some(unsafe { self.src.get_unchecked(self.idx) })
    }

    /// Same as [Reader::peek] but will return an `Result` instead a `Option`.
    #[inline]
    pub fn lookahead(&self) -> Result<&u8> {
        self.peek()
            .ok_or_else(|| self.error::<&u8>(ErrorKind::Eof).unwrap_err())
    }

    /// Get `n` bytes without advance.
    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_ref().get(..n)
    }

    /// Get `n` bytes and advance.
    pub fn read_n(&mut self, n: usize) -> Result<&[u8]> {
        let start = self.idx;
        for _ in 0..n {
            self.next();
        }
        let end = self.idx;

        Ok(&self.src[start..end])
    }

    /// Call the clousore `func` for each next byte in the slice, and process the element
    /// (without advance the iterator) while it returs `true`.
    ///
    /// # Returns
    ///
    /// The byte right after last peek-ed.
    pub fn peek_while<F>(&self, func: F) -> Option<&u8>
    where
        F: Fn(&u8) -> bool,
    {
        let src = self.as_ref();
        for (i, &b) in src.iter().enumerate() {
            if !func(&b) {
                return src.get(i);
            }
        }
        None
    }
    /// Read a `u32` number from the slice.
    ///
    /// This method reads until an invalid digit is found.
    pub fn read_u32(&mut self) -> Result<u32> {
        let bytes = digits!(self);
        let digits = unsafe { str::from_utf8_unchecked(bytes) };

        match digits.parse() {
            Ok(num) => Ok(num),
            Err(_) => self.error(ErrorKind::Num),
        }
    }

    /// Read a `u16` number from the slice.
    ///
    /// This method reads until an invalid digit is found.
    pub fn read_u16(&mut self) -> Result<u16> {
        let bytes = digits!(self);
        let digits = unsafe { str::from_utf8_unchecked(bytes) };

        match digits.parse() {
            Ok(num) => Ok(num),
            Err(_) => self.error(ErrorKind::Num),
        }
    }

    /// `read_while()` will call the `func` clousure for each element in the slice and advance
    /// while the closure returns `true`.
    ///
    /// # Returns
    ///
    /// It will return the (`start..end`) range, that is, the first and last index
    /// processed in the slice.
    pub fn read_while<F>(&mut self, func: F) -> Range<usize>
    where
        F: Fn(&u8) -> bool,
    {
        let start = self.idx;
        while let Ok(Some(_)) = self.read_if(&func) {}
        let end = self.idx;

        Range { start, end }
    }

    /// Read next byte if equals to `b`.
    ///
    /// # Errors
    ///
    /// This method will return an error if the byte is not equal to `b`.
    ///
    /// If the slice reached the end, then an error will also be returned.
    pub fn must_read(&mut self, b: &u8) -> Result<()> {
        let Some(n) = self.peek() else {
            return self.error(ErrorKind::Eof);
        };
        if b != n {
            return self.error(ErrorKind::Char {
                expected: *b,
                found: *n,
            });
        }
        self.next();
        Ok(())
    }

    /// Same as [Reader::read_while] but will return the slice of bytes converted to a string slice.
    ///
    /// # Safety
    ///
    /// Caller must ensures that `func` valid that bytes are valid UTF-8.
    pub unsafe fn read_as_str<F>(&mut self, func: F) -> &'a str
    where
        F: Fn(&u8) -> bool,
    {
        let bytes = read_while!(self, &func);

        // SAFETY: the caller must guarantee that the `func` valid that bytes are valid UTF-8.
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

    pub fn read_number_as_str(&mut self) -> &'a str {
        let mut range = self.read_while(is_digit);

        while let Some(&b'.') = self.peek() {
            self.next();
            range.end = self.read_while(is_digit).end;
        }

        let num_slice = &self.src[range];

        unsafe { str::from_utf8_unchecked(num_slice) }
    }

    /// Call the `func` closure for next byte and read it if the closure returns `true`.
    ///
    /// # Returns
    ///
    /// The byte readed.
    pub fn read_if<F>(&mut self, func: F) -> Result<Option<&u8>>
    where
        F: FnOnce(&u8) -> bool,
    {
        let Some(b) = self.peek() else {
            return self.error(ErrorKind::Eof);
        };
        if !func(b) {
            return Ok(None);
        }

        Ok(self.next())
    }

    pub fn cur_is_some_and<F>(&self, func: F) -> bool
    where
        F: FnOnce(&u8) -> bool,
    {
        self.peek().is_some_and(func)
    }

    #[inline(always)]
    fn advance(&mut self) -> &'a u8 {
        let byte = unsafe { self.src.get_unchecked(self.idx) };
        if byte == &b'\n' {
            self.pos.col = 1;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
        self.idx += 1;

        byte
    }

    fn error<T>(&self, kind: ErrorKind) -> Result<T> {
        Err(Error {
            kind,
            line: self.pos.line,
            col: self.pos.col,
            src: self.src,
        })
    }
}

impl ToString for Reader<'_> {
    fn to_string(&self) -> String {
        String::from_utf8_lossy(&self.src[self.idx..]).into()
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
}

#[derive(Debug, PartialEq)]
pub struct Error<'a> {
    pub kind: ErrorKind,
    pub line: usize,
    pub col: usize,
    pub src: &'a [u8],
}

impl<'a> AsRef<[u8]> for Reader<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.src[self.idx..]
    }
}

impl<'a> Iterator for Reader<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let byte = self.advance();
        if self.idx == self.len {
            self.finished = true;
        }
        Some(byte)
    }
}

#[cfg(test)]
mod tests {
    use crate::util::{is_alphabetic, is_newline, is_space};

    use super::*;

    #[test]
    fn test_peek() {
        let src = "Hello, world!".as_bytes();
        let reader = Reader::new(src);

        assert_eq!(reader.peek(), Some(&b'H'));
        assert_eq!(reader.peek_n(6), Some("Hello,".as_bytes()));

        let b = reader.peek_while(|b| b != &b',');
        assert_eq!(b, Some(&b','));
    }

    #[test]
    fn test_read() {
        let src = "Input to\r\nread".as_bytes();
        let mut reader = Reader::new(src);

        let range = reader.read_while(|b| b == &b'I');
        assert_eq!(&src[range], "I".as_bytes());

        let range = reader.read_while(is_alphabetic);
        assert_eq!(&src[range], "nput".as_bytes());

        let range = reader.read_while(is_space);
        assert_eq!(&src[range], " ".as_bytes());

        assert_eq!(reader.read_if(is_alphabetic), Ok(Some(&b't')));
        assert_eq!(reader.next(), Some(&b'o'));

        let range = reader.read_while(is_newline);
        assert_eq!(&src[range], "\r\n".as_bytes());

        assert_eq!(reader.pos.line, 2);
        assert_eq!(reader.pos.col, 1);
    }

    #[test]
    fn test_read_num() {
        let mut reader = Reader::new("12345".as_bytes());
        assert_eq!(reader.read_num(), Ok(12345));

        let mut reader = Reader::new("NaN".as_bytes());
        assert!(reader.read_num::<u32>().is_err());
        assert_eq!(reader.as_ref(), b"NaN");

        let mut reader = Reader::new("9123Test".as_bytes());
        assert_eq!(reader.read_num(), Ok(9123));
        assert_eq!(reader.as_ref(), b"Test");
    }

    #[test]
    fn test_lookahead() {
        let mut reader = Reader::new("Hello".as_bytes());

        assert_eq!(reader.lookahead(), Ok(&b'H'));
        reader.next();
        assert_eq!(reader.lookahead(), Ok(&b'e'));
        reader.next();
        assert_eq!(reader.lookahead(), Ok(&b'l'));

        reader.read_while(|_| true);

        assert!(reader.lookahead().is_err());
    }
}
