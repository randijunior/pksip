use std::str;

use crate::{parser::Result, scanner::Scanner, util::is_newline};

use crate::headers::SipHeader;

use super::space;

/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
pub struct Timestamp {
    time: f32,
    delay: Option<f32>,
}

impl<'a> SipHeader<'a> for Timestamp {
    const NAME: &'static str = "Timestamp";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let time = scanner.read_num()?;
        space!(scanner);
        let delay = if scanner.peek().is_some_and(|b| !is_newline(b)) {
            Some(scanner.read_num()?)
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

        assert_eq!(timestamp.time, 54.0);
    }
}
