use std::cell::UnsafeCell;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub(crate) line: usize,
    pub(crate) col: usize,
    pub(crate) idx: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position {
            line: 1,
            col: 1,
            idx: 0,
        }
    }
}

pub struct Cursor<'a> {
    input: &'a [u8],
    pos: UnsafeCell<Position>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Cursor {
            input,
            pos: UnsafeCell::new(Position::default()),
        }
    }

    #[inline(always)]
    fn pos(&self) -> &Position {
        unsafe { &*self.pos.get() }
    }

    #[inline(always)]
    fn pos_mut(&self) -> &mut Position {
        unsafe { &mut *self.pos.get() }
    }

    #[inline(always)]
    pub fn is_eof(&self) -> bool {
        self.pos().idx == self.input.len()
    }

    pub fn peek(&self) -> Option<u8> {
        let pos = self.pos();

        match self.input.get(pos.idx) {
            Some(&b) => Some(b),
            None => None,
        }
    }

    fn consume(&self) -> u8 {
        let state = self.pos_mut();
        let byte = self.input[state.idx];

        state.idx += 1;
        if byte == b'\n' {
            state.col = 1;
            state.line += 1;
        } else {
            state.col += 1;
        }
        byte
    }

    pub fn advance(&self) -> Option<u8> {
        if !self.is_eof() {
            Some(self.consume())
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn parts(&self, start: usize, end: usize) -> &[u8] {
        assert!(end <= self.input.len());
        &self.input[start..end]
    }
    
    #[inline]
    pub fn index(&self) -> usize {
        let state = self.pos();

        state.idx
    }

    pub fn position(&self) -> Position {
        let state = self.pos();

        Position {
            line: state.line,
            col: state.col,
            idx: state.idx,
        }
    }

    fn as_slice(&self) -> &[u8] {
        let state = self.pos();

        &self.input[state.idx..]
    }

    pub fn input(&self) -> &[u8] {
        self.input
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
