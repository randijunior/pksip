use std::str;

use reader::util::is_newline;
use reader::{space, Reader};

use crate::parser::Result;

use crate::headers::SipHeader;



/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Timestamp<'a> {
    time: &'a str,
    delay: Option<&'a str>,
}

impl<'a> SipHeader<'a> for Timestamp<'a> {
    const NAME: &'static str = "Timestamp";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let time = reader.scan_number_as_str();
        space!(reader);
        let has_delay = reader.peek().is_some_and(|b| !is_newline(b));

        let delay = if has_delay {
            Some(reader.scan_number_as_str())
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
        let mut reader = Reader::new(src);
        let timestamp = Timestamp::parse(&mut reader);
        let timestamp = timestamp.unwrap();

        assert_eq!(timestamp.time, "54.0");
    }
}
