use std::{cell::UnsafeCell, marker::PhantomData};

use crate::util::is_newline;

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

    Predicate,

    Delimiter,
}
#[derive(Debug, PartialEq)]
pub struct ReaderError<'a> {
    pub(crate) kind: ErrorKind,
    pub(crate) pos: Position,
    pub(crate) input: &'a [u8],
}

pub struct ReaderState<'a> {
    begin: *const u8,
    end: *const u8,
    cur: *const u8,
    start_line: *const u8,
    line: usize,
    maker: std::marker::PhantomData<&'a ()>,
}

impl<'a> ReaderState<'a> {
    pub fn from_input(input: &'a [u8]) -> Self {
        let begin = input.as_ptr();
        ReaderState {
            cur: begin,
            begin,
            end: unsafe { begin.add(input.len()) },
            start_line: begin,
            line: 1,
            maker: PhantomData,
        }
    }
}
/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    state: UnsafeCell<ReaderState<'a>>,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        let reader_state = ReaderState::from_input(input);

        InputReader {
            input,
            state: UnsafeCell::new(reader_state),
        }
    }
    // Safety: the caller must ensure that there are no mutable references that
    // point to contents of the state.
    unsafe fn get_state(&self) -> &ReaderState {
        &*self.state.get()
    }

    fn cur(&self) -> *const u8 {
        unsafe { self.get_state() }.cur
    }

    unsafe fn next(&self, state: &mut ReaderState) -> ReaderResult<u8> {
        let byte = *state.cur;
        state.cur = state.cur.add(1);

        if is_newline(byte) {
            state.start_line = state.cur;
            state.line += 1;
        }

        Ok(byte)
    }

    fn is_eof(&self, state: &ReaderState) -> bool {
        state.cur >= state.end
    }

    pub fn read(&self) -> ReaderResult<u8> {
        unsafe {
            let shared = self.get_state();
            if self.is_eof(shared) {
                return Err(self.error(ErrorKind::EndOfInput));
            }
        }
        unsafe {
            let exclusive = &mut *self.state.get();
            self.next(exclusive)
        }
    }

    pub fn read_n(&self, n: usize) -> ReaderResult<&[u8]> {
        let start = self.cur();
        for _ in 0..n {
            self.read()?;
        }
        let end = self.cur();

        Ok(unsafe { self.slice_from_parts(start, end) })
    }

    pub fn peek(&self) -> Option<u8> {
        unsafe {
            let state = self.get_state();
            if self.is_eof(state) {
                return None;
            }
            Some(*state.cur) 
        }
    }

    fn get_col(&self, state: &ReaderState) -> usize {
        let cur = state.cur as usize;
        let st_line = state.start_line as usize;

        debug_assert!(cur >= st_line);

        cur - st_line
    }

    pub fn error(&self, kind: ErrorKind) -> ReaderError {
        let state = unsafe { self.get_state() };
        ReaderError {
            kind,
            pos: Position {
                line: state.line,
                col: self.get_col(state),
            },
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
        let start = self.cur();
        let mut next = self.peeking_next(&predicate);
        while let Ok(_) = next {
            next = self.peeking_next(&predicate);
        }
        let end = self.cur();

        Ok(unsafe { self.slice_from_parts(start, end) })
    }

    fn peeking_next<P>(&self, predicate: P) -> ReaderResult<u8>
    where
        P: Fn(u8) -> bool,
    {
        if let Some(n) = self.peek() {
            if predicate(n) {
                self.read()
            } else {
                Err(self.error(ErrorKind::Predicate))
            }
        } else {
            Err(self.error(ErrorKind::EndOfInput))
        }
    }

    pub fn peek_for_match(&self, i: &[u8]) -> Option<&u8> {
        self.as_slice().iter().find(|&byte| i.contains(byte))
    }

    pub fn next_if_eq(&self, expected: u8) -> ReaderResult<u8> {
        if let Some(byte) = self.peek() {
            if byte == expected {
                self.read()
            } else {
                Err(self.error(ErrorKind::Predicate))
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
            let state = self.get_state();
            self.slice_from_parts(state.cur, state.end)
        }
    }

    unsafe fn slice_from_parts(&self, start: *const u8, end: *const u8) -> &[u8] {
        assert!(start <= end);
        core::slice::from_raw_parts(start, end as usize - start as usize)
    }
}
