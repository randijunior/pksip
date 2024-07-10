use crate::util::is_newline;

type ReaderResult<T> = Result<T, ReaderError>;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    line: usize,
    col: usize,
}
/// Errors that can occur while reading the input.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    TagMismatch,
    /// End of file reached.
    EndOfInput,
    /// Insufficient input for the requested operation.
    OutOfInput,

    DelimiterNotFound,
}
#[derive(Debug, PartialEq)]
pub struct ReaderError {
    kind: ErrorKind,
    pos: Position,
}

impl ReaderError {
    pub fn line(&self) -> usize {
        self.pos.line
    }
    pub fn col(&self) -> usize {
        self.pos.col
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    iterator: std::iter::Peekable<std::slice::Iter<'a, u8>>,
    pub position: Position,
    idx: usize,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    ///
    /// # Arguments
    ///
    /// * `input` - A byte slice representing the input.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input,
            iterator: input.iter().peekable(),
            position: Position { line: 1, col: 0 },
            idx: 0,
        }
    }

    pub fn read(&mut self) -> ReaderResult<&u8> {
        self.next()
    }

    pub fn remaing(&self) -> &[u8] {
        &self.input[self.idx..]
    }

    fn update_cursor(&mut self, byte: &u8) {
        self.idx = self.idx + 1;
        self.position.col += 1;
        if is_newline(*byte) {
            self.position.line += 1;
            self.position.col = 1;
        }
    }

    fn next(&mut self) -> ReaderResult<&u8> {
        if let Some(byte) = self.iterator.next() {
            self.update_cursor(byte);
            Ok(byte)
        } else {
            Err(self.error(ErrorKind::EndOfInput))
        }
    }

    pub fn peek(&mut self) -> Option<&&u8> {
        self.iterator.peek()
    }

    fn peeking_next<P>(&mut self, predicate: P) -> ReaderResult<Option<&u8>>
    where
        P: FnOnce(&&u8) -> bool,
    {
        if let Some(n) = self.peek() {
            if predicate(&n) {
                Ok(Some(self.next()?))
            } else {
                Ok(None)
            }
        } else {
            Err(self.error(ErrorKind::EndOfInput))
        }
    }

    pub fn is_eof(&self) -> bool {
        self.idx == self.input.len()
    }

    fn next_if_eq(&mut self, expected: &&u8) -> ReaderResult<&u8> {
        match self.iterator.next_if_eq(expected) {
            Some(byte) => {
                self.update_cursor(byte);
                Ok(byte)
            }
            None => {
                if self.is_eof() {
                    Err(self.error(ErrorKind::OutOfInput))
                } else {
                    Err(self.error(ErrorKind::TagMismatch))
                }
            }
        }
    }

    /// Matches the input against the given prefix.
    pub fn tag(&mut self, tag: &[u8]) -> ReaderResult<&[u8]> {
        let len = tag.len();

        if len > self.iterator.len() {
            return Err(self.error(ErrorKind::OutOfInput));
        }
        let start = self.idx;
        for byte in tag.iter() {
            self.next_if_eq(&&byte)?;
        }
        let end = self.idx;

        Ok(&self.input[start..end])
    }

    fn advance_n(&mut self, n: usize) -> ReaderResult<()> {
        for _ in 0..n {
            self.next()?;
        }
        Ok(())
    }

    /// Reads `n` bytes from the input.
    pub fn read_n(&mut self, n: usize) -> ReaderResult<&[u8]> {
        if n > self.iterator.len() {
            return Err(self.error(ErrorKind::OutOfInput));
        }
        let start = self.idx;
        self.advance_n(n)?;
        let end = self.idx;

        Ok(&self.input[start..end])
    }

    fn take_while<P>(&mut self, predicate: P) -> ReaderResult<&[u8]>
    where
        P: Fn(&&u8) -> bool,
    {
        let start = self.idx;
        let mut next = self.peeking_next(&predicate);

        while let Ok(Some(_)) = next {
            next = self.peeking_next(&predicate);
        }
        let end = self.idx;

        Ok(&self.input[start..end])
    }

    /// Reads bytes from the input while the predicate is true.
    pub fn read_while(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> ReaderResult<&[u8]> {
        self.take_while(|n| predicate(**n))
    }

    /// Reads bytes from the input until the predicate returns `true`.
    pub fn read_until(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> ReaderResult<&[u8]> {
        self.take_while(|n| !predicate(**n))
    }

    pub fn read_next_if<P>(&mut self, predicate: P) -> ReaderResult<Option<&u8>>
    where
        P: FnOnce(u8) -> bool,
    {
        if let Some(c) = self.peeking_next(|c| predicate(**c))? {
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    pub fn read_until_byte(&mut self, byte: u8) -> ReaderResult<&[u8]> {
        self.read_until(|b| b == byte)
    }

    pub fn peek_for_match(&self, i: &[u8]) -> Option<&u8> {
        for byte in self.remaing().iter() {
            if i.contains(&byte) {
                return Some(byte);
            }
        }
        None
    }

    pub fn error(&self, kind: ErrorKind) -> ReaderError {
        ReaderError {
            kind,
            pos: self.position,
        }
    }
}
