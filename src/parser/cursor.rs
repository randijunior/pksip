use std::{cell::UnsafeCell, marker::PhantomData};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

pub struct State<'a> {
    begin: *const u8,
    end: *const u8,
    curptr: *const u8,
    start_line: *const u8,
    line: usize,
    idx: usize,
    maker: PhantomData<&'a ()>,
}

pub struct Cursor<'a>(UnsafeCell<State<'a>>);

impl<'a> Cursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        let begin = input.as_ptr();
        Cursor(UnsafeCell::new(State {
            curptr: begin,
            begin,
            end: unsafe { begin.add(input.len()) },
            start_line: begin,
            line: 1,
            maker: PhantomData,
            idx: 0
        }))
    }

    #[inline(always)]
    fn state(&self) -> &State {
        unsafe { &*self.0.get() }
    }

    #[inline(always)]
    fn state_mut(&self) -> &mut State<'a> {
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
        state.idx += 1;
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

    pub fn index(&self) -> usize {
        let state = self.state();

        state.idx
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

pub trait CursorIter {
    type Item;

    fn next(&self) -> Option<Self::Item>;
}

impl<'a> CursorIter for Cursor<'a> {
    type Item = u8;

    fn next(&self) -> Option<Self::Item> {
        self.advance()
    }
}


pub(crate) unsafe fn slice_from_parts<'a>(start: *const u8, end: *const u8) -> &'a [u8] {
    assert!(start <= end);
    core::slice::from_raw_parts(start, end as usize - start as usize)
}