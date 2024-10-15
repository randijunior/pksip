use core::str;

use crate::{
    scanner::Scanner,
    macros::read_while,
    parser::Result,
    util::{is_float, is_newline},
};

use super::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for Timestamp<'a> {
    const NAME: &'static [u8] = b"Timestamp";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let time = read_while!(scanner, is_float);
        let time = unsafe { str::from_utf8_unchecked(time) };
        let delay = if scanner.peek().is_some_and(|&b| !is_newline(b)) {
            let delay = read_while!(scanner, is_float);
            Some(unsafe { str::from_utf8_unchecked(delay) })
        } else {
            None
        };
        Ok(Timestamp { time, delay })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"54\r\n";
        let mut scanner = Scanner::new(src);
        let timestamp = Timestamp::parse(&mut scanner);
        let timestamp = timestamp.unwrap();

        assert_eq!(timestamp, Timestamp { delay: None, time: "54" });
    }
     
}