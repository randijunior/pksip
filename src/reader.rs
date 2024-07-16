use crate::util::{Cursor, Position};

type Result<'a, T> = std::result::Result<T, ReaderError<'a>>;
/// Errors that can occur while reading the input.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    /// The tag did not match the expected value.
    Tag,
    /// End of file reached.
    EndOfInput,
    /// Insufficient input for the requested operation.
    OutOfInput,
}
#[derive(Debug, PartialEq)]
pub struct ReaderError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) pos: Position,
    pub(crate) input: &'a [u8],
}
/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    cursor: Cursor<'a>,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input,
            cursor: Cursor::new(input),
        }
    }

    fn next(&self) -> Result<u8> {
        match self.cursor.advance() {
            Some(byte) => Ok(byte),
            None => Err(self.error(ErrorKind::EndOfInput)),
        }
    }

    pub fn read(&self) -> Result<u8> {
        self.next()
    }

    pub fn peek(&self) -> Option<u8> {
        self.cursor.peek()
    }

    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_slice().get(..n)
    }

    fn position(&self) -> Position {
        self.cursor.position()
    }

    pub fn error(&self, kind: ErrorKind) -> ReaderError {
        ReaderError {
            kind,
            pos: self.position(),
            input: self.input,
        }
    }

    pub fn read_until_b(&self, byte: u8) -> Result<&[u8]> {
        self.read_until(|b| b == byte)
    }

    pub fn read_until<P>(&self, predicate: P) -> Result<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        self.read_while(|n| !predicate(n))
    }

    pub fn read_while<P>(&self, predicate: P) -> Result<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        let start = self.cursor.cursor();
        let mut next = self.next_if(&predicate);
        while let Ok(Some(_)) = next {
            next = self.next_if(&predicate);
        }
        let end = self.cursor.cursor();

        Ok(unsafe { crate::util::slice_from_parts(start, end) })
    }

    fn next_if<P>(&self, predicate: P) -> Result<Option<u8>>
    where
        P: Fn(u8) -> bool,
    {
        if let Some(n) = self.peek() {
            if predicate(n) {
                Ok(self.read().ok())
            } else {
                Ok(None)
            }
        } else {
            Err(self.error(ErrorKind::EndOfInput))
        }
    }

    pub fn tag(&self, tag: &[u8]) -> Result<&[u8]> {
        let len = tag.len();
        if let Some(bytes) = self.peek_n(len) {
            for i in 0..len {
                if bytes[i] != tag[i] {
                    return Err(self.error(ErrorKind::Tag));
                }
                self.read()?;
            }
            Ok(bytes)
        } else {
            Err(self.error(ErrorKind::OutOfInput))
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.cursor.as_ref()
    }
}
