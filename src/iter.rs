use std::ops::Range;

type Result<'a, T> = std::result::Result<T, ByteReaderError<'a>>;
/// Errors that can occur while reading the input.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    Tag,
    /// End of file reached.
    Eof,
    /// Insufficient input for the requested operation.
    OutOfInput,
}

#[derive(Debug, PartialEq)]
pub struct ByteReaderError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) input: &'a [u8],
}
#[derive(Debug)]
pub(crate) struct ByteReader<'a> {
    pub(crate) input: &'a [u8],
    finished: bool,
    len: usize,
    line: usize,
    col: usize,
    idx: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        ByteReader {
            input,
            len: input.len(),
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
        Some(&self.input[self.idx])
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        self.idx
    }

    pub fn tag(&mut self, tag: &[u8]) -> Result<Range<usize>> {
        let start = self.idx();
        for &expected in tag {
            let Some(&byte) = self.peek() else {
                return self.error(ErrorKind::OutOfInput);
            };
            if byte != expected {
                return self.error(ErrorKind::Tag);
            }
            self.next();
        }
        let end = self.idx();
        Ok(start..end)
    }

    pub fn read_while<F>(&mut self, func: F) -> Result<Range<usize>>
    where
        F: Fn(u8) -> bool,
    {
        let start = self.idx();
        let mut next = self.read_if(&func);
        while let Some(_) = next {
            next = self.read_if(&func);
        }
        let end = self.idx();
        Ok(start..end)
    }

    #[inline(always)]
    pub fn col(&self) -> usize {
        self.col
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
    pub fn line(&self) -> usize {
        self.line
    }

    #[inline(always)]
    pub fn bump(&mut self, byte: &u8) {
        self.idx += 1;
        if *byte == b'\n' {
            self.col = 1;
            self.line += 1;
        } else {
            self.col += 1;
        }
    }

    pub fn error<T>(
        &self,
        kind: ErrorKind,
    ) -> std::result::Result<T, ByteReaderError> {
        Err(ByteReaderError {
            kind,
            line: self.line(),
            col: self.col(),
            input: self.input,
        })
    }
}

impl<'a> AsRef<[u8]> for ByteReader<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.input[self.idx()..]
    }
}

impl<'a> Iterator for ByteReader<'a> {
    type Item = &'a u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let byte = &self.input[self.idx];
        self.bump(byte);
        if self.idx + 1 == self.len {
            self.finished = true;
        }
        Some(byte)
    }
}
