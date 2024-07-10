use std::marker::PhantomData;

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
    begin: *const u8,
    end: *const u8,
    st_line: *const u8,
    cur: *const u8,
    line: u32,
    maker: std::marker::PhantomData<&'a ()>,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    ///
    /// # Arguments
    ///
    /// * `input` - A byte slice representing the input.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        let begin = input.as_ptr();
        let end = unsafe { begin.add(input.len()) };
        let st_line = begin;
        let line = 1;
        let cur = begin;
        
        InputReader {
            cur,
            begin,
            end,
            st_line,
            line,
            maker: PhantomData,
        }
    }

    pub fn read(&mut self) -> Option<u8> {
        if self.cur >= self.end {
            None
        } else {
            unsafe {
                self.cur = self.cur.add(1);
                let byte = *self.cur;
                Some(byte)
            }
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        assert!(self.cur <= self.end);
        let slice = unsafe {
            core::slice::from_raw_parts(
                self.cur,
                self.end as usize - self.cur as usize,
            )
        };

        slice
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader() {
        let mut reader = InputReader::new(b"a");

        reader.read();

        println!("{:#?}", std::str::from_utf8(reader.as_slice()));
    }
}
