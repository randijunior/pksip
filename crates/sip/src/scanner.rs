use std::{ops::Range, result};

type Result<'a, T> = std::result::Result<T, ScannerError<'a>>;
/// Errors that can occur while reading the src.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// End of file reached.
    Eof,
}

#[derive(Debug, PartialEq)]
pub struct ScannerError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) src: &'a [u8],
}


#[derive(Debug)]
pub struct Scanner<'a> {
    pub(crate) src: &'a [u8],
    finished: bool,
    len: usize,
    line: usize,
    col: usize,
    idx: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        Scanner {
            src,
            len: src.len(),
            finished: false,
            line: 1,
            col: 1,
            idx: 0,
        }
    }

    
    #[inline]
    pub fn idx(&self) -> usize {
        self.idx
    }

    
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.finished
    }

    
    pub fn peek(&self) -> Option<&u8> {
        if self.is_eof() {
            return None;
        }

        Some(&self.src[self.idx])
    }

    
    pub(crate) fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_ref().get(..n)
    }


    pub(crate) fn peek_while<F>(&self, func: F) -> Range<usize>
    where
        F: Fn(&u8) -> bool,
    {
        let start = self.idx;
        let iter = self.as_ref().iter();
        let iter = iter.take_while(|&b| func(b));
        let end = start + iter.count();

        Range { start, end }
    }


    pub(crate) fn read_while<F>(&mut self, func: F) -> Range<usize>
    where
        F: Fn(&u8) -> bool,
    {
        let start = self.idx;
        let mut b = self.read_if(&func);

        while let Ok(Some(_)) = b {
            b = self.read_if(&func);
        }
        let end = self.idx;

        Range { start, end }
    }

    
    pub(crate) fn read_if<F>(&mut self, func: F) -> Result<Option<&u8>>
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

    fn error<T>(&self, kind: ErrorKind) -> result::Result<T, ScannerError> {
        Err(ScannerError {
            kind,
            line: self.line,
            col: self.col,
            src: self.src,
        })
    }
}

impl<'a> AsRef<[u8]> for Scanner<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.src[self.idx..]
    }
}

impl<'a> Iterator for Scanner<'a> {
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
        let scanner = Scanner::new(src);

        assert_eq!(scanner.peek(), Some(&b'H'));
        assert_eq!(scanner.peek_n(6), Some("Hello,".as_bytes()));

        let range = scanner.peek_while(is_alphabetic);
        assert_eq!(&src[range], "Hello".as_bytes());
    }

    #[test]
    fn test_read() {
        let src = "Input to\r\nread".as_bytes();
        let mut scanner = Scanner::new(src);

        let range = scanner.read_while(|b| b == &b'I');
        assert_eq!(&src[range], "I".as_bytes());

        let range = scanner.read_while(is_alphabetic);
        assert_eq!(&src[range], "nput".as_bytes());

        let range = scanner.read_while(is_space);
        assert_eq!(&src[range], " ".as_bytes());

        assert_eq!(scanner.read_if(is_alphabetic), Ok(Some(&b't')));
        assert_eq!(scanner.next(), Some(&b'o'));

        let range = scanner.read_while(is_newline);
        assert_eq!(&src[range], "\r\n".as_bytes());

        assert_eq!(scanner.line, 2);
        assert_eq!(scanner.col, 1);
    }
}
