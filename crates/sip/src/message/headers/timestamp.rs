use core::str;

use crate::{
    bytes::Bytes,
    macros::read_while,
    parser::Result,
    util::{is_newline, maybe_a_number},
};

use crate::headers::SipHeader;

/// Describes when the `UAC` sent the request to the `UAS`.
pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeader<'a> for Timestamp<'a> {
    const NAME: &'static str = "Timestamp";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let time = read_while!(bytes, maybe_a_number);
        let time = unsafe { str::from_utf8_unchecked(time) };
        let delay = if bytes.peek().is_some_and(|b| !is_newline(b)) {
            let delay = read_while!(bytes, maybe_a_number);
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
        let mut bytes = Bytes::new(src);
        let timestamp = Timestamp::parse(&mut bytes);
        let timestamp = timestamp.unwrap();

        assert_eq!(timestamp.time, "54");
    }
}
