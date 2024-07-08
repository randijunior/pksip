use itertools::PeekingNext;

use crate::util::is_newline;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    line: usize,
    col: usize,
}
/// Errors that can occur while reading the input.
#[derive(Debug)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    TagMismatch,
    /// End of file reached.
    EndOfInput,
    /// Invalid UTF-8 encountered in the input.
    InvalidUtf8,
    /// Insufficient input for the requested operation.
    OutOfInput,
}
#[derive(Debug)]
pub struct ReaderError {
    pub(crate) err: ErrorKind,
    pub(crate) pos: Option<Position>,
}

/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    iterator: std::slice::Iter<'a, u8>,
    pub position: Position,
    idx: usize,
    remaing: &'a [u8],
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
            iterator: input.iter(),
            position: Position { line: 1, col: 1 },
            idx: 0,
            remaing: input,
        }
    }

    /// Reads the next byte from the input and updates the position.
    /// Returns [`ReaderError::EndOfInput`] if the end of input is reached.
    pub fn read(&mut self) -> Result<&u8, ReaderError> {
        Ok(self.next()?)
    }

    fn next(&mut self) -> Result<&u8, ReaderError> {
        let c = self.iterator.next();
        if let Some(char) = c {
            self.update_pos(char);
            return Ok(char);
        } else {
            return Err(ReaderError {
                err: ErrorKind::EndOfInput,
                pos: Some(self.position.clone()),
            });
        }
    }

    fn update_pos(&mut self, c: &u8) {
        self.remaing = self.iterator.as_slice();
        self.idx = self.idx + 1;
        self.position.col += 1;
        if is_newline(*c) {
            self.position.line += 1;
            self.position.col = 1;
        }
    }

    /// Matches the input against the given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A byte slice representing the prefix to match.
    ///
    /// If the prefix does not match, returns [`ReaderError::TagMismatch`],
    /// if there are not enough bytes left in the input returns [`ReaderError::OutOfInput`].
    pub fn prefix(&mut self, prefix: &[u8]) -> Result<&[u8], ReaderError> {
        let len = prefix.len();
        let slice = self.remaing;

        if len >= slice.len() {
            return Err(ReaderError {
                err: ErrorKind::EndOfInput,
                pos: Some(self.position.clone()),
            });
        }
        let (c, ..) = slice.split_at(len);
        for i in 0..len {
            let a = c[i];
            let b = prefix[i];

            if a != b {
                return Err(ReaderError {
                    err: ErrorKind::TagMismatch,
                    pos: Some(self.position.clone()),
                });
            }
            self.read()?;
        }

        Ok(c)
    }

    /// Reads `n` bytes from the input.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of bytes to read.
    pub fn read_n(&mut self, n: usize) -> Result<&[u8], ReaderError> {
        if n > self.remaing.len() {
            return Err(ReaderError {
                err: ErrorKind::OutOfInput,
                pos: Some(self.position.clone()),
            });
        }
        let start = self.idx;
        for _ in 0..n {
            self.next()?;
        }
        let end = self.idx;

        Ok(&self.input[start..end])
    }

    fn take_while_matching(
        &mut self,
        cb: impl Fn(&&u8) -> bool,
    ) -> Result<&[u8], ReaderError> {
        let start = self.idx;

        while let Some(c) = self.iterator.peeking_next(&cb) {
            self.update_pos(c)
        }

        Ok(&self.input[start..self.idx])
    }

    /// Reads bytes from the input while the predicate is true.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A closure that takes a byte and returns `true` if the byte should be read.
    pub fn read_while(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<&[u8], ReaderError> {
        self.take_while_matching(|next| predicate(**next))
    }

    /// Reads bytes from the input until the predicate returns `true`.
    ///
    /// This function will continue reading bytes from the input until it encounters
    /// a byte for which the predicate returns `true`. The byte that matches the predicate
    /// will not be included in the returned slice, and the reader will be positioned just before
    /// that byte.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A closure that takes a byte and returns `true` when the reading should stop.
    pub fn read_until(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<&[u8], ReaderError> {
        self.take_while_matching(|next| !predicate(**next))
    }

    /// Reads bytes from the input until the predicate returns `true`, consuming the matching byte.
    ///
    /// Same as [`InputReader::read_until`], but the reader will consume the bytes that matches the predicate.
    pub fn read_until_and_consume(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<&[u8], ReaderError> {
        let start = self.idx;
        self.read_until(&predicate)?;
        let end = self.idx;

        self.read_while(predicate)?;

        Ok(&self.input[start..end])
    }
}
