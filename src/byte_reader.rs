use std::ops::Range;

type ReaderResult<'a, T> = std::result::Result<T, ReaderError<'a>>;
/// Errors that can occur while reading the src.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    Tag,
    /// End of file reached.
    Eof,
    /// Insufficient src for the requested operation.
    OutOfInput,
}

#[derive(Debug, PartialEq)]
pub struct ReaderError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) src: &'a [u8],
}
#[derive(Debug)]
pub(crate) struct ByteReader<'a> {
    pub(crate) src: &'a [u8],
    finished: bool,
    len: usize,
    line: usize,
    col: usize,
    idx: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(src: &'a [u8]) -> Self {
        ByteReader {
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

    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_ref().get(..n)
    }

    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.as_ref().iter()
    }

    pub fn peek_while<F>(&self, func: F) -> Range<usize>
    where
        F: Fn(u8) -> bool,
    {
        let mut end = self.idx;
        for _ in self.iter().take_while(|&&b| func(b)) {
            end += 1;
        }
        self.idx..end
    }

    pub fn read_tag(&mut self, tag: &[u8]) -> ReaderResult<Range<usize>> {
        let start = self.idx;
        for &expected in tag {
            let Some(&byte) = self.peek() else {
                return self.error(ErrorKind::OutOfInput);
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
        while let Some(_) = next {
            next = self.read_if(&func);
        }
        let end = self.idx;

        start..end
    }

    pub fn read_if_eq(&mut self, expected: u8) -> Option<&u8> {
        self.read_if(|b| b == expected)
    }

    pub fn read_if<F>(&mut self, func: F) -> Option<&u8>
    where
        F: FnOnce(u8) -> bool,
    {
        match self.peek() {
            Some(&b) => {
                if func(b) {
                    self.next()
                } else {
                    None
                }
            }
            None => None,
        }
    }

    #[inline(always)]
    pub fn advance(&mut self) -> &'a u8 {
        let byte = &self.src[self.idx];
        if *byte == b'\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }
        self.idx += 1;

        byte
    }

    pub fn error<T>(&self, kind: ErrorKind) -> Result<T, ReaderError> {
        Err(ReaderError {
            kind,
            line: self.line,
            col: self.col,
            src: self.src,
        })
    }
}

impl<'a> AsRef<[u8]> for ByteReader<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.src[self.idx..]
    }
}

impl<'a> Iterator for ByteReader<'a> {
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
