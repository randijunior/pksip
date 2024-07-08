use crate::util::is_newline;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    line: usize,
    col: usize,
}
/// Errors that can occur while reading the input.
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    TagMismatch,
    /// End of file reached.
    EndOfInput,
    /// Invalid UTF-8 encountered in the input.
    InvalidUtf8,
    /// Insufficient input for the requested operation.
    OutOfInput,

    DelimiterNotFound,
}
#[derive(Debug, PartialEq)]
pub struct ReaderError {
    kind: ErrorKind,
    pos: Position,
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

    pub fn peek(&mut self) -> Option<&&u8> {
        self.iterator.peek()
    }

    fn peeking_next(
        &mut self,
        predicate: impl Fn(&&u8) -> bool,
        check_utf8: bool,
    ) -> Result<Option<&u8>, ReaderError> {
        if let Some(n) = self.peek() {
            if predicate(n) {
                Ok(self.next(check_utf8).ok())
            } else {
                Ok(None)
            }
        } else {
            Err(self.error(ErrorKind::EndOfInput))
        }
    }

    /// Reads the next byte from the input and updates the position.
    /// Returns [`ReaderError::EndOfInput`] if the end of input is reached.
    pub fn read(&mut self) -> Result<&u8, ReaderError> {
        Ok(self.next(false)?)
    }

    pub fn read_utf8(&mut self) -> Result<&u8, ReaderError> {
        Ok(self.next(true)?)
    }

    fn next(&mut self, utf8_check: bool) -> Result<&u8, ReaderError> {
        let c = self.iterator.next();
        if let Some(char) = c {
            self.update_pos(char);
            if utf8_check {
                self.validate_utf8()?;
            }
            return Ok(char);
        } else {
            return Err(self.error(ErrorKind::EndOfInput));
        }
    }

    pub fn error(&self, kind: ErrorKind) -> ReaderError {
        ReaderError {
            kind,
            pos: self.position,
        }
    }

    fn update_pos(&mut self, c: &u8) {
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
    /// If the prefix does not match, returns [`ReaderError::PrefixMismatch`],
    /// if there are not enough bytes left in the input returns [`ReaderError::OutOfInput`].
    pub fn tag(&mut self, tag: &[u8]) -> Result<&[u8], ReaderError> {
        let len = tag.len();
        let slice = &self.input[self.idx..];

        if len >= slice.len() {
            return Err(self.error(ErrorKind::OutOfInput));
        }
        let (c, ..) = slice.split_at(len);
        for i in 0..len {
            if c[i] != tag[i] {
                return Err(self.error(ErrorKind::TagMismatch));
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
        let remaing = &self.input[self.idx..];
        if n > remaing.len() {
            return Err(self.error(ErrorKind::OutOfInput));
        }
        let start = self.idx;
        for _ in 0..n {
            self.next(false)?;
        }

        Ok(&self.input[start..self.idx])
    }

    fn take_while_matching(
        &mut self,
        cb: impl Fn(&&u8) -> bool,
        utf8_check: bool,
    ) -> Result<&[u8], ReaderError> {
        let start = self.idx;
        let mut next = self.peeking_next(&cb, utf8_check)?;

        while next.is_some() {
            let peek_res = self.peeking_next(&cb, utf8_check);
            match peek_res {
                Ok(n) => next = n,
                Err(_) => break,
            }
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
        self.take_while_matching(|next| predicate(**next), false)
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
        self.take_while_matching(|next| !predicate(**next), false)
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

    pub fn read_while_utf8(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<&str, ReaderError> {
        let start = self.idx;

        self.take_while_matching(|next| predicate(**next), true)?;

        Ok(unsafe { std::str::from_utf8_unchecked(&self.input[start..self.idx]) })
    }

    pub fn read_until_and_consume_str(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<&str, ReaderError> {
        let start = self.idx;
        self.take_while_matching(|next| !predicate(**next), true)?;
        let end = self.idx;

        self.read_while(&predicate)?;

        Ok(unsafe { std::str::from_utf8_unchecked(&self.input[start..end]) })
    }

    fn is_eof(&self) -> bool {
        self.idx == self.input.len()
    }

    fn read_next_if(
        &mut self,
        predicate: impl Fn(u8) -> bool,
    ) -> Result<Option<&u8>, ReaderError> {
        if let Some(c) = self.peeking_next(|c| predicate(**c), false)? {
            Ok(Some(c))
        } else {
            Ok(None)
        }
    }

    pub fn read_between(
        &mut self,
        beg_delim: u8,
        end_delim: u8,
    ) -> Result<&[u8], ReaderError> {
        let next = self.read_next_if(|c| c == beg_delim)?;

        if next.is_none() {
            return Err(self.error(ErrorKind::DelimiterNotFound));
        }
        let start = self.idx;
        self.take_while_matching(|next| **next != end_delim, false)?;
        let end = self.idx;

        Ok(&self.input[start..end])
    }

    pub fn validate_utf8(&mut self) -> Result<(), ReaderError> {
        let (i, ..) = self.input.split_at(self.idx);
        if let Err(_) = std::str::from_utf8(i) {
            Err(self.error(ErrorKind::InvalidUtf8))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_bytes() {
        let invalid_bytes = b"abc\x80\x80defg";
        let mut reader = InputReader::new(invalid_bytes);

        assert!(reader.read().is_ok());
        assert!(reader.read().is_ok());
        assert!(reader.read().is_ok());

        assert_eq!(
            reader.read_utf8(),
            Err(ReaderError {
                kind: ErrorKind::InvalidUtf8,
                pos: Position { line: 1, col: 4 }
            })
        );
    }
}
