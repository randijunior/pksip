use std::ops::Range;

type ScannerResult<'a, T> = std::result::Result<T, ScannerError<'a>>;
/// Errors that can occur while reading the src.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    Tag,
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
pub(crate) struct Scanner<'a> {
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

    pub fn peek(&self) -> Option<&u8> {
        if self.finished {
            return None;
        }
        Some(&self.src[self.idx])
    }

    #[inline]
    pub fn is_eof(&self) -> bool {
        self.finished
    }

    pub fn read_n(&mut self, n: usize) -> ScannerResult<Range<usize>> {
        let start = self.idx;
        for _ in 0..n {
            if let None = self.next() {
                return self.error(ErrorKind::Eof);
            }
        }
        let end = self.idx;
        Ok(start..end)
    }

    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_ref().get(..n)
    }

    pub fn peek_while<F>(&self, func: F) -> Range<usize>
    where
        F: Fn(u8) -> bool,
    {
        let mut end = self.idx;
        let iter = self.as_ref().iter().take_while(|&&b| func(b));
        for _ in iter {
            end += 1;
        }
        self.idx..end
    }

    pub fn read_tag(&mut self, tag: &[u8]) -> ScannerResult<Range<usize>> {
        let start = self.idx;
        for &expected in tag {
            let Some(&byte) = self.peek() else {
                return self.error(ErrorKind::Eof);
            };
            if byte != expected {
                return self.error(ErrorKind::Tag);
            }
            self.next();
        }
        let end = self.idx;
        Ok(start..end)
    }

    pub fn read_while<F>(&mut self, func: F) -> Range<usize>
    where
        F: Fn(u8) -> bool,
    {
        let start = self.idx;
        let mut next = self.read_if(&func);
        while let Ok(Some(_)) = next {
            next = self.read_if(&func);
        }
        let end = self.idx;

        start..end
    }

    pub fn read_if_eq(&mut self, expected: u8) -> ScannerResult<Option<&u8>> {
        self.read_if(|b| b == expected)
    }

    pub fn read_if<F>(&mut self, func: F) -> ScannerResult<Option<&u8>>
    where
        F: FnOnce(u8) -> bool,
    {
        match self.peek() {
            Some(&b) => {
                if func(b) {
                    Ok(self.next())
                } else {
                    Ok(None)
                }
            }
            None => self.error(ErrorKind::Eof),
        }
    }

    #[inline(always)]
    pub fn advance(&mut self) -> &'a u8 {
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

    pub fn error<T>(&self, kind: ErrorKind) -> Result<T, ScannerError> {
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
mod test {
    use crate::util::{is_alphabetic, is_newline, is_space};

    use super::*;

    #[test]
    fn test_peek() {
        let src = "Hello, world!".as_bytes();
        let scanner = Scanner::new(src);

        assert_eq!(scanner.peek(), Some(&b'H'));
        assert_eq!(scanner.peek_n(6), Some("Hello,".as_bytes()));
        assert_eq!(scanner.peek_while(is_alphabetic), Range { start: 0, end: 5 });
    }

    #[test]
    fn test_tag() {
        let src = "This is an test!".as_bytes();
        let mut scanner = Scanner::new(src);

        assert_eq!(scanner.read_tag(b"This"), Ok(Range { start: 0, end: 4 }));
        assert_eq!(scanner.read_tag(b" is"), Ok(Range { start: 4, end: 7 }));
        assert_eq!(
            scanner.read_tag(b"not exist!"),
            Err(ScannerError {
                kind: ErrorKind::Tag,
                line: 1,
                col: 8,
                src: src
            })
        );
        assert_eq!(
            scanner.read_tag(b" an test!"),
            Ok(Range { start: 7, end: 16 })
        );
        assert_eq!(
            scanner.read_tag(b"end!"),
            Err(ScannerError {
                kind: ErrorKind::Eof,
                line: 1,
                col: 17,
                src: src
            })
        );
    }

    #[test]
    fn test_read() {
        let src = "Input to\r\nread".as_bytes();
        let mut scanner = Scanner::new(src);

        assert_eq!(
            scanner.read_while(|b| b == b'I'),
            Range { start: 0, end: 1 }
        );
        assert_eq!(
            scanner.read_while(is_alphabetic),
            Range { start: 1, end: 5 }
        );
        assert_eq!(scanner.read_while(is_space), Range { start: 5, end: 6 });
        assert_eq!(scanner.read_if(|b| b == b't'), Ok(Some(&b't')));
        assert_eq!(scanner.read_if_eq(b'o'), Ok(Some(&b'o')));
        assert_eq!(
            scanner.read_while(is_newline),
            Range { start: 8, end: 10 }
        );

        assert_eq!(scanner.line, 2);
        assert_eq!(scanner.col, 1);

        assert_eq!(scanner.read_n(4), Ok(Range { start: 10, end: 14 }));
    }
}
