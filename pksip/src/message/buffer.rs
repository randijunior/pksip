use std::io;
use std::io::Write;

#[derive(Debug)]
pub struct Buffer(Vec<u8>);

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Buffer> for Box<[u8]> {
    fn from(b: Buffer) -> Self {
        b.0.into()
    }
}

impl Buffer {
    #[inline(always)]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    #[inline]
    pub fn extend_from_slice(&mut self, other: &[u8]) {
        self.0.extend_from_slice(other)
    }
}

impl Write for Buffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
