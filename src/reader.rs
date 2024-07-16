use std::{cell::UnsafeCell, marker::PhantomData};

type ReaderResult<'a, T> = Result<T, ReaderError<'a>>;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
}
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

pub struct Cursor<'a> {
    begin: *const u8,
    end: *const u8,
    curptr: *const u8,
    start_line: *const u8,
    line: usize,
    maker: std::marker::PhantomData<&'a ()>,
}

unsafe fn slice_from_parts<'a>(start: *const u8, end: *const u8) -> &'a [u8] {
    assert!(start <= end);
    core::slice::from_raw_parts(start, end as usize - start as usize)
}

impl<'a> Cursor<'a> {
    pub fn from_input(input: &'a [u8]) -> Self {
        let begin = input.as_ptr();
        Cursor {
            curptr: begin,
            begin,
            end: unsafe { begin.add(input.len()) },
            start_line: begin,
            line: 1,
            maker: PhantomData,
        }
    }

    pub fn is_eof(&self) -> bool {
        self.curptr >= self.end
    }
}

impl<'a> AsRef<[u8]> for Cursor<'a> {
    fn as_ref(&self) -> &[u8] {
        unsafe { slice_from_parts(self.curptr, self.end) }
    }
}
/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    cursor: UnsafeCell<Cursor<'a>>,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input,
            cursor: UnsafeCell::new(Cursor::from_input(input)),
        }
    }

    fn next(&self) -> ReaderResult<u8> {
        unsafe {
            let cursor = &mut *self.cursor.get();
            let byte = *cursor.curptr;
            cursor.curptr = cursor.curptr.add(1);

            if byte == b'\n' {
                cursor.start_line = cursor.curptr;
                cursor.line += 1;
            }

            Ok(byte)
        }
    }

    pub fn read(&self) -> ReaderResult<u8> {
        unsafe {
            let cursor = &*self.cursor.get();
            if cursor.is_eof() {
                return Err(self.error(ErrorKind::EndOfInput));
            }
        }
        self.next()
    }

    pub fn read_n(&self, n: usize) -> ReaderResult<&[u8]> {
        let start = unsafe {
            let cursor = &*self.cursor.get();
            cursor.curptr
        };
        for _ in 0..n {
            self.read()?;
        }
        let end = unsafe {
            let cursor = &*self.cursor.get();
            cursor.curptr
        };

        Ok(unsafe { slice_from_parts(start, end) })
    }

    pub fn peek(&self) -> Option<u8> {
        unsafe {
            let cursor = &*self.cursor.get();

            return if cursor.is_eof() {
                None
            } else {
                Some(*cursor.curptr)
            };
        }
    }

    fn get_position(&self) -> Position {
        let (curptr, st_line, line) = unsafe {
            let cursor = &*self.cursor.get();

            (cursor.curptr, cursor.start_line, cursor.line)
        };
        let cur = curptr as usize;
        let st_line = st_line as usize;

        debug_assert!(cur >= st_line);

        Position {
            line,
            col: cur - st_line,
        }
    }

    pub fn error(&self, kind: ErrorKind) -> ReaderError {
        ReaderError {
            kind,
            pos: self.get_position(),
            input: self.input,
        }
    }

    pub fn read_until_b(&self, byte: u8) -> ReaderResult<&[u8]> {
        self.read_until(|b| b == byte)
    }

    pub fn read_until<P>(&self, predicate: P) -> ReaderResult<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        self.read_while(|n| !predicate(n))
    }

    pub fn read_while<P>(&self, predicate: P) -> ReaderResult<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        let start = unsafe {
            let cursor = &*self.cursor.get();
            cursor.curptr
        };
        let mut next = self.peeking_next(&predicate);
        while let Ok(Some(_)) = next {
            next = self.peeking_next(&predicate);
        }
        let end = unsafe {
            let cursor = &*self.cursor.get();
            cursor.curptr
        };

        Ok(unsafe { slice_from_parts(start, end) })
    }

    fn peeking_next<P>(&self, predicate: P) -> ReaderResult<Option<u8>>
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

    pub fn peek_for_match(&self, i: &[u8]) -> Option<&u8> {
        self.as_slice().iter().find(|&byte| i.contains(byte))
    }

    pub fn next_if_eq(&self, expected: u8) -> ReaderResult<Option<u8>> {
        if let Some(byte) = self.peek() {
            if byte == expected {
                Ok(self.read().ok())
            } else {
                Ok(None)
            }
        } else {
            Err(self.error(ErrorKind::OutOfInput))
        }
    }

    pub fn tag(&self, tag: &[u8]) -> Result<&[u8], ReaderError> {
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

    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_slice().get(..n)
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let cursor = &*self.cursor.get();
            cursor.as_ref()
        }
    }
}
