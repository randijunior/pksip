#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) idx: usize,
}

type Result<'a, T> = std::result::Result<T, CursorError<'a>>;
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
pub struct CursorError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) pos: Position,
    pub(crate) input: &'a [u8],
}
#[derive(Debug)]
pub struct Cursor<'a> {
    pub(crate) input: &'a [u8],
    pub(crate) pos: Position,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Cursor {
            input,
            pos: Position {
                line: 1,
                col: 1,
                idx: 0,
            },
        }
    }

    pub fn peek(&self) -> Option<u8> {
        let pos = self.pos;

        match self.input.get(pos.idx) {
            Some(&b) => Some(b),
            None => None,
        }
    }

    pub fn read_while(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<(usize, usize)> {
        let start = self.pos.idx;
        let mut next = self.read_if(&predicate);
        while let Some(_) = next {
            next = self.read_if(&predicate);
        }
        let end = self.pos.idx;

        Ok((start, end))
    }

    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(self.as_ref()).to_string()
    }

    pub fn read_if(&mut self, func: impl Fn(u8) -> bool) -> Option<u8> {
        self.peek()
            .and_then(|n| if func(n) { self.next() } else { None })
    }

    #[inline(always)]
    pub fn is_eof(&self) -> bool {
        self.pos.idx == self.input.len()
    }

    pub fn error(&self, kind: ErrorKind) -> CursorError<'a> {
        CursorError {
            kind,
            pos: self.pos,
            input: self.input,
        }
    }
}

impl<'a> AsRef<[u8]> for Cursor<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.input[self.pos.idx..]
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_eof() {
            return None;
        }
        let byte = self.input[self.pos.idx];

        self.pos.idx += 1;
        if byte == b'\n' {
            self.pos.col = 1;
            self.pos.line += 1;
        } else {
            self.pos.col += 1;
        }
        Some(byte)
    }
}
