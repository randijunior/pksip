use core::str;

use crate::{bytes::Bytes, parser::Result, util::is_newline};

use crate::headers::SipHeader;

use super::space;

/// Describes when the `UAC` sent the request to the `UAS`.
pub struct Timestamp {
    time: f32,
    delay: Option<f32>,
}

impl<'a> SipHeader<'a> for Timestamp {
    const NAME: &'static str = "Timestamp";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let time = bytes.parse_num()?;
        space!(bytes);
        let delay = if bytes.peek().is_some_and(|b| !is_newline(b)) {
            Some(bytes.parse_num()?)
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

        assert_eq!(timestamp.time, 54.0);
    }
}
