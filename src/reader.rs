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
    TagMismatch,
    /// End of file reached.
    EndOfInput,
    /// Insufficient input for the requested operation.
    OutOfInput,

    DelimiterNotFound,
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
/// A struct for reading and parsing input byte by byte.
pub struct InputReader<'a> {
    input: &'a [u8],
    state: UnsafeCell<ReaderState<'a>>,
}

impl<'a> InputReader<'a> {
    /// Creates a new `InputReader` from the given input slice.
    pub fn new(input: &'a [u8]) -> InputReader<'a> {
        let begin = input.as_ptr();
        let end = unsafe { begin.add(input.len()) };
        let cur = begin;
        let start_line = begin;

        InputReader {
            input,
            state: UnsafeCell::new(ReaderState {
                cur,
                begin,
                end,
                start_line,
                line: 1,
                maker: PhantomData,
            }),
        }
    }

    unsafe fn get_state(&self) -> &ReaderState {
        &*self.state.get()
    }

    unsafe fn get_state_mut(&self) -> &mut ReaderState<'a> {
        &mut *self.state.get()
    }

    unsafe fn cur(&self) -> *const u8 {
        let state = self.get_state();
        state.cur
    }

    pub fn read(&self) -> ReaderResult<u8> {
        let state = unsafe { self.get_state_mut() };
        if state.cur >= state.end {
            return Err(self.error(ErrorKind::EndOfInput));
        } else {
            unsafe {
                let byte = *state.cur;
                state.cur = state.cur.add(1);

                if is_newline(byte) {
                    state.start_line = state.cur;
                    state.line += 1;
                }
                Ok(byte)
            }
        }
    }

    pub fn read_n(&self, n: usize) -> ReaderResult<&[u8]> {
        let start = unsafe { self.cur() };
        for _ in 0..n {
            self.read()?;
        }
        let end = unsafe { self.cur() };

        Ok(unsafe { self.slice_from_parts(start, end) })
    }

    pub fn peek(&self) -> Option<u8> {
        let state = unsafe { self.get_state() };
        if state.cur >= state.end {
            return None;
        }
        unsafe { Some(*state.cur) }
    }

    fn get_col(&self, state: &ReaderState) -> usize {
        let cur = state.cur as usize;
        let st_line = state.start_line as usize;

        assert!(cur >= st_line);

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
            return Err(self.error(ErrorKind::EndOfInput));
        }
    }

    pub fn read_while<P>(&self, predicate: P) -> ReaderResult<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        let start = unsafe { self.cur() };
        let mut next = self.peeking_next(&predicate);
        while let Ok(Some(_)) = next {
            next = self.peeking_next(&predicate);
        }
        let end = unsafe { self.cur() };

        Ok(unsafe { self.slice_from_parts(start, end) })
    }

    pub fn read_until_b(&self, byte: u8) -> ReaderResult<&[u8]> {
        self.read_until(|b| b == byte)
    }

    pub fn peek_for_match(&self, i: &[u8]) -> Option<&u8> {
        for byte in self.as_slice().iter() {
            if i.contains(&byte) {
                return Some(byte);
            }
        }
        None
    }

    pub fn read_next_if_eq(&self, expected: u8) -> ReaderResult<Option<u8>> {
        match self.peek() {
            Some(byte) => {
                if byte == expected {
                    Ok(self.read().ok())
                } else {
                    Ok(None)
                }
            }
            None => Err(self.error(ErrorKind::OutOfInput)),
        }
    }

    pub fn tag(&self, tag: &[u8]) -> Result<&[u8], ReaderError> {
        let len = tag.len();
        let slc = self.peek_n(len);

        match slc {
            Some(bytes) => {
                for i in 0..len {
                    if bytes[i] != tag[i] {
                        return Err(self.error(ErrorKind::TagMismatch));
                    }
                    self.read()?;
                }
                Ok(bytes)
            }
            None => Err(self.error(ErrorKind::OutOfInput)),
        }
    }

    pub fn peek_n(&self, n: usize) -> Option<&[u8]> {
        self.as_slice().get(..n)
    }

    pub fn read_until<P>(&self, predicate: P) -> ReaderResult<&[u8]>
    where
        P: Fn(u8) -> bool,
    {
        self.read_while(|n| !predicate(n))
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
