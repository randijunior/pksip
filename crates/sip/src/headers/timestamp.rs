use std::str;

use crate::{parser::Result, scanner::Scanner, util::is_newline};

use crate::headers::SipHeader;

use super::space;

/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeader<'a> for Timestamp<'a> {
    const NAME: &'static str = "Timestamp";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let time = scanner.scan_number_as_str();
        space!(scanner);
        let has_delay = scanner.peek().is_some_and(|b| !is_newline(b));

        let delay = if has_delay {
            Some(scanner.scan_number_as_str())
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
        let src = b"54.0 1.5\r\n";
        let mut scanner = Scanner::new(src);
        let timestamp = Timestamp::parse(&mut scanner);
        let timestamp = timestamp.unwrap();

        assert_eq!(timestamp.time, "54.0");
    }
}
