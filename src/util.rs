use std::{cell::UnsafeCell, marker::PhantomData};

#[inline(always)]
pub fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[inline(always)]
pub fn is_space(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

#[inline(always)]
pub fn is_newline(c: u8) -> bool {
    c == b'\r' || c == b'\n'
}

#[inline(always)]
pub fn is_alphabetic(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

pub struct CursorState<'a> {
    begin: *const u8,
    end: *const u8,
    curptr: *const u8,
    start_line: *const u8,
    line: usize,
    maker: std::marker::PhantomData<&'a ()>,
}

pub struct Cursor<'a>(UnsafeCell<CursorState<'a>>);

impl<'a> Cursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let begin = input.as_ptr();
        Cursor(UnsafeCell::new(CursorState {
            curptr: begin,
            begin,
            end: unsafe { begin.add(input.len()) },
            start_line: begin,
            line: 1,
            maker: PhantomData,
        }))
    }

    #[inline(always)]
    fn state(&self) -> &CursorState {
        unsafe { &*self.0.get() }
    }

    #[inline(always)]
    fn state_mut(&self) -> &mut CursorState<'a> {
        unsafe { &mut *self.0.get() }
    }

    pub fn is_eof(&self) -> bool {
        let state = self.state();

        state.curptr >= state.end
    }

    pub fn peek(&self) -> Option<u8> {
        let state = self.state();
        if state.curptr < state.end {
            unsafe {
                Some(*state.curptr)
            }
        } else {
            None
        }
    }

    unsafe fn consume(&self) -> u8 {
        let state = self.state_mut();

        let byte = *state.curptr;
        state.curptr = state.curptr.add(1);
        if byte == b'\n' {
            state.start_line = state.curptr;
            state.line += 1;
        }
        byte
    }

    pub fn advance(&self) -> Option<u8> {
        if !self.is_eof() {
            unsafe {
                Some(self.consume())
            }
        } else {
            None
        }
    }

    pub fn cursor(&self) -> *const u8 {
        let state = self.state();

        state.curptr
    }

    pub fn position(&self) -> Position {
        let state = self.state();

        let cur = state.curptr as usize;
        let st_line = state.start_line as usize;

        debug_assert!(cur >= st_line);

        Position {
            line: state.line,
            col: cur - st_line,
        }
    }

    fn as_slice(&self) -> &[u8] {
        let state = self.state();
        unsafe {
            slice_from_parts(state.curptr, state.end)
        }
    }
}

impl<'a> AsRef<[u8]> for Cursor<'a> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

pub(crate) unsafe fn slice_from_parts<'a>(start: *const u8, end: *const u8) -> &'a [u8] {
    assert!(start <= end);
    core::slice::from_raw_parts(start, end as usize - start as usize)
}
