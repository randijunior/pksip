#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) idx: usize,
}

type Result<'a, T> = std::result::Result<T, ByteReaderError<'a>>;
type Range = (usize, usize);
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
    pos: Position,
}

impl<'a> ByteReader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        ByteReader {
            input,
            len: input.len(),
            finished: false,
            pos: Position {
                line: 1,
                col: 1,
                idx: 0,
            },
        }
    }

    pub fn peek(&self) -> Option<&u8> {
        if self.finished {
            return None;
        }
        Some(&self.input[self.pos.idx])
    }

    pub fn tag(&mut self, tag: &[u8]) -> Result<Range> {
        let start = self.idx();
        for &expected in tag {
            let Some(&byte) = self.next() else {
                return Err(self.error(ErrorKind::OutOfInput));
            };
            if byte != expected {
                return Err(self.error(ErrorKind::Tag));
            }
        }
        let end = self.idx();
        Ok((start, end))
    }

    #[inline(always)]
    fn advance(&mut self, byte: &u8) {
        self.pos.idx += 1;
        if *byte == b'\n' {
            self.pos.col = 1;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
    }

    #[inline(always)]
    pub fn idx(&self) -> usize {
        self.pos.idx
    }

    pub fn read_while<F>(&mut self, func: F) -> Result<Range>
    where
        F: Fn(u8) -> bool,
    {
        let start = self.idx();
        let mut next = self.read_if(&func);
        while let Some(_) = next {
            next = self.read_if(&func);
        }
        let end = self.idx();
        Ok((start, end))
    }

    #[inline(always)]
    pub fn col(&self) -> usize {
        self.pos.col
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(self.as_ref()).to_string()
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
        self.pos.line
    }

    pub fn error(&self, kind: ErrorKind) -> ByteReaderError<'a> {
        ByteReaderError {
            kind,
            line: self.line(),
            col: self.col(),
            input: self.input,
        }
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
        let byte = &self.input[self.pos.idx];
        self.advance(byte);

        if self.pos.idx + 1 == self.len {
            self.finished = true;
        }
        Some(byte)
    }
}
