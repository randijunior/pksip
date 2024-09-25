use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::read_while,
    parser::Result,
    util::{is_float, is_newline},
};

use super::SipHeaderParser;

pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for Timestamp<'a> {
    const NAME: &'static [u8] = b"Timestamp";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let time = read_while!(reader, is_float);
        let time = unsafe { str::from_utf8_unchecked(time) };
        let delay = if reader.peek().is_some_and(|&b| !is_newline(b)) {
            let delay = read_while!(reader, is_float);
            Some(unsafe { str::from_utf8_unchecked(delay) })
        } else {
            None
        };
        Ok(Timestamp { time, delay })
    }
}
