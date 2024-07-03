use itertools::PeekingNext;

pub struct InputReader<'a> {
    input: &'a [u8],
    iterator: std::slice::Iter<'a, u8>,
    position: Position,
    idx: usize
}

#[derive(Debug, Clone, Copy)]
struct Position {
    line: usize,
    col: usize,
}

#[derive(Debug)]
pub enum ParseError {
    TagMismatch,
    Eof,
    Other(&'static str)
}

impl<'a> InputReader<'a> {
    pub fn new(i: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input: i,
            iterator: i.iter(),
            position: Position { line: 1, col: 1 },
            idx: 0
        }
    }
    
    pub fn tag(&mut self, tag: &[u8]) -> Result<&[u8], ParseError> {
        let len = tag.len();
        let slice = self.iterator.as_slice();

        if len >= slice.len() {
            return Err(ParseError::Eof)
        }
        let (c, ..) = slice.split_at(len);
        if c != tag {
            return Err(ParseError::TagMismatch)
        }
        self.read_n(len)?;

        Ok(c)
    }

    pub fn read_n(&mut self, n: usize) -> Result<&[u8], ParseError> {
        let start = self.idx;
        for _ in 0..n {
            self.next()?;
        }
        let end = self.idx;

        Ok(&self.input[start..end])
    }


    pub fn as_slice(&self) -> &[u8] {
        self.iterator.as_slice()
    }

    pub fn read_while<F>(&mut self, predicate: F) -> Result<&[u8], ParseError> 
     where F: Fn(u8) -> bool {
        let start = self.idx;

        while let Some(c) = self.iterator.peeking_next(|next| predicate(**next)) {
            self.update_pos(c)
        }

        Ok(&self.input[start..self.idx])
    }

    pub fn read(&mut self) -> Result<u8, ParseError>  {
        let c = self.next()?;

        Ok(*c)
    }

    fn update_pos(&mut self, c: &u8) {
        self.idx = self.idx + 1;
        self.position.col += 1;
        if self.is_new_line(c) {
            self.position.line += 1;
            self.position.col = 1;
        }
    }


    fn next(&mut self) -> Result<&u8, ParseError> {
        let c = self.iterator.next();
        if let Some(char) = c {
            self.update_pos(char);
            return Ok(char);
        } else {
            return Err(ParseError::Eof);
        }
    }

    #[inline]
    fn is_new_line(&self, c: &u8) -> bool {
        *c == 0x0A
    }

    #[inline]
    pub fn is_digit(c: u8) -> bool {
      c >= 0x30 && c <= 0x39
    }

    #[inline]
    pub fn is_space(c: u8) -> bool {
        c == b' ' || c == b'\t'
    }
}

