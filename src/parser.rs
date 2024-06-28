pub struct Position {
    line: usize,
    col: usize,
}

impl Position {
    fn inc_line(&mut self) {
        self.line += 1;
    }

    fn inc_col(&mut self) {
        self.col += 1;
    }
}

impl Default for Position {
    fn default() -> Self {
        Position { line: 1, col: 1 }
    }
}

pub trait InputLen {
    fn input_len(&self) -> usize;
}


impl<T, const N: usize> InputLen for &[T; N] {
    fn input_len(&self) -> usize {
        self.len()
    }
}



pub struct InputReader<'a> {
    input: &'a [u8],
    iterator: std::slice::Iter<'a, u8>,
    idx: usize,
    position: Position,
}


impl<'a> InputReader<'a> {
    pub fn new(i: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input: i,
            iterator: i.iter(),
            idx: 0,
            position: Position::default(),
        }
    }

    pub fn tag<I: InputLen + Clone>(&self, tag: I) -> &[u8] {
        let len = tag.input_len();
        let t = tag.clone();

        todo!()
    }

    pub fn peek_n(&self, len: usize) -> &[u8] {
        let remaining = self.iterator.as_slice();

        todo!()
    }

    pub fn peek(&mut self) -> Option<&u8> {
        let c = self.iterator.next();
        if let Some(char) = c {
            self.position.inc_col();
            self.idx += 1;
            if self.is_new_line(char) {
                self.position.inc_line();
            }
        }
        c
    }

    fn is_new_line(&self, c: &u8) -> bool {
        *c == 0x0D || *c == 0x0A
    }

    fn is_space(&self, c: &u8) -> bool {
        *c == b' ' || *c == b'\t'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let reader = InputReader::new("SIP/".as_bytes());

        let sip = reader.tag(b"SIP");

    }
}
