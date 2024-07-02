use core::fmt;

pub trait InputLen {
    fn input_len(&self) -> usize;
}

impl<T, const N: usize> InputLen for &[T; N] {
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<'a> InputLen for &'a str {
    fn input_len(&self) -> usize {
        self.len()
    }
}
#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: String) -> Self {
        ParseError { message }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error parsing {}", self.message)
    }
}

#[derive(Clone, Debug)]
pub struct Position {
    line: usize,
    col: usize,
}

impl Default for Position {
    fn default() -> Self {
        Position { line: 1, col: 1 }
    }
}

pub trait Compare<T> {
    fn compare(&self, t: T) -> bool;
}

impl<'a, 'b, const N: usize> Compare<&'b [u8]> for &'a [u8; N] {
    fn compare(&self, t: &'b [u8]) -> bool {
        *self == t
    }
}

impl<'a, 'b> Compare<&'b [u8]> for &'a str {
    fn compare(&self, t: &[u8]) -> bool {
        self.as_bytes() == t
    }
}

pub struct InputReader<'a> {
    input: &'a [u8],
    iterator: std::slice::Iter<'a, u8>,
    position: Position,
}

impl<'a> InputReader<'a> {
    pub fn new(i: &'a [u8]) -> InputReader<'a> {
        InputReader {
            input: i,
            iterator: i.iter(),
            position: Position::default(),
        }
    }

    pub fn reader_get<T>(&self, chr: T) -> Result<&[u8], ParseError>
    where
        T: InputLen + Clone + for<'i> Compare<&'i [u8]>,
    {
        let len = chr.input_len();
        let t = chr.clone();
        let i = self.peek_n(len);

        match t.compare(i) {
            true => Ok(i),
            false => Err(ParseError::new( "pattern not found!".to_string())),
        }
    }

    pub fn peek_n(&self, len: usize) -> &[u8] {
        let remaining = self.iterator.as_slice();
        if len > remaining.len() {
            return remaining;
        }
        return &remaining[..len];
    }

    pub fn peek(&mut self) -> Option<&u8> {
        let c = self.iterator.next();
        if let Some(char) = c {
            self.position.col += 1;
            if self.is_new_line(char) {
                self.position.line += 1;
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

        let sip = reader.reader_get(b"SA");
        println!("{}", std::str::from_utf8(sip.unwrap()).unwrap());
    }
}
