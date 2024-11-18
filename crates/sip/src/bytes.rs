use std::ops::Range;
use std::str;

use crate::macros::read_while;

type Result<'a, T> = std::result::Result<T, BytesError<'a>>;

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
pub struct BytesError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) src: &'a [u8],
}

/// Reading byte slice while keep the line and column.
#[derive(Debug)]
pub struct Bytes<'a> {
    /// The input bytes slice to be read.
    pub(crate) src: &'a [u8],
    /// Indicates if the reading is complete.
    finished: bool,
    /// Total length of the input slice.
    len: usize,
    /// Current line.
    line: usize,
    /// Current column.
    col: usize,
    /// Current index.
    idx: usize,
}

impl<'a> Bytes<'a> {
    /// Create a `Bytes` from a byte slice.
    ///
    /// The `line` and `col` will always start from 1.
    pub fn new(src: &'a [u8]) -> Self {
        Bytes {
            src,
            len: src.len(),
            finished: false,
            line: 1,
            col: 1,
            idx: 0,
        }
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
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.finished
    }

    /// Get next byte without advance.
    pub fn peek(&self) -> Option<&u8> {
        if self.is_eof() {
            return None;
        }

        Some(&self.src[self.idx])
    }

    /// Same as [Bytes::peek] but will return an `Result` instead a `Option`.
    pub fn lookahead(&self) -> Result<&u8> {
        self.peek()
            .ok_or_else(|| self.error::<&u8>(ErrorKind::Eof).unwrap_err())
    }

    /// Get `n` bytes without advance.
    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_ref().get(..n)
    }

    /// Call the clousore `func` for each next byte in the slice, and process the element
    /// (without advance the iterator) while it returs `true`.
    ///
    /// # Returns
    ///
    /// The number of processed elements.
    pub fn peek_while<F>(&self, func: F) -> usize
    where
        F: Fn(&u8) -> bool,
    {
        let iter = self.as_ref().iter();
        let iter = iter.take_while(|&b| func(b));
        let processed = self.idx + iter.count();

        processed
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
    pub fn must_read(&mut self, b: u8) -> Result<()> {
        let Some(&n) = self.peek() else {
            return self.error(ErrorKind::Eof);
        };
        if b != n {
            return self.error(ErrorKind::Char {
                expected: b,
                found: n,
            });
        }
        self.next();
        Ok(())
    }

    /// Same as [Bytes::read_while] but will return the slice of bytes converted to a string slice.
    /// 
    /// # Safety
    /// 
    /// Caller must ensures that `func` valid that bytes are valid UTF-8.
    pub unsafe fn read_and_convert_to_str<F>(&mut self, func: F) -> &'a str
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

    #[inline(always)]
    fn advance(&mut self) -> &'a u8 {
        let byte = &self.src[self.idx];
        if byte == &b'\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }
        self.idx += 1;

        byte
    }

    fn error<T>(&self, kind: ErrorKind) -> Result<T> {
        Err(BytesError {
            kind,
            line: self.line,
            col: self.col,
            src: self.src,
        })
    }
}

impl<'a> AsRef<[u8]> for Bytes<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.src[self.idx..]
    }
}

impl<'a> Iterator for Bytes<'a> {
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
        let bytes = Bytes::new(src);

        assert_eq!(bytes.peek(), Some(&b'H'));
        assert_eq!(bytes.peek_n(6), Some("Hello,".as_bytes()));

        let range = bytes.peek_while(is_alphabetic);
        assert_eq!(range, "Hello".len());
    }

    #[test]
    fn test_read() {
        let src = "Input to\r\nread".as_bytes();
        let mut bytes = Bytes::new(src);

        let range = bytes.read_while(|b| b == &b'I');
        assert_eq!(&src[range], "I".as_bytes());

        let range = bytes.read_while(is_alphabetic);
        assert_eq!(&src[range], "nput".as_bytes());

        let range = bytes.read_while(is_space);
        assert_eq!(&src[range], " ".as_bytes());

        assert_eq!(bytes.read_if(is_alphabetic), Ok(Some(&b't')));
        assert_eq!(bytes.next(), Some(&b'o'));

        let range = bytes.read_while(is_newline);
        assert_eq!(&src[range], "\r\n".as_bytes());

        assert_eq!(bytes.line, 2);
        assert_eq!(bytes.col, 1);
    }

    #[test]
    fn test_read_num() {
        let mut bytes = Bytes::new("12345".as_bytes());
        assert_eq!(bytes.read_num(), Ok(12345));

        let mut bytes = Bytes::new("NaN".as_bytes());
        assert!(bytes.read_num::<u32>().is_err());
        assert_eq!(bytes.as_ref(), b"NaN");

        let mut bytes = Bytes::new("9123Test".as_bytes());
        assert_eq!(bytes.read_num(), Ok(9123));
        assert_eq!(bytes.as_ref(), b"Test");
    }

    #[test]
    fn test_lookahead() {
        let mut bytes = Bytes::new("Hello".as_bytes());

        assert_eq!(bytes.lookahead(), Ok(&b'H'));
        bytes.next();
        assert_eq!(bytes.lookahead(), Ok(&b'e'));
        bytes.next();
        assert_eq!(bytes.lookahead(), Ok(&b'l'));

        bytes.read_while(|_| true);

        assert!(bytes.lookahead().is_err());
    }
}
