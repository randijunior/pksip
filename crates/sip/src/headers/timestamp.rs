use std::{fmt, str};

use reader::util::is_newline;
use reader::{space, Reader};

use crate::internal::ArcStr;
use crate::parser::Result;

use crate::headers::SipHeader;

/// The `Timestamp` SIP header.
///
/// Describes when the `UAC` sent the request to the `UAS`.
#[derive(Debug, PartialEq, Eq)]
pub struct Timestamp {
    time: ArcStr,
    delay: Option<ArcStr>,
}

impl SipHeader<'_> for Timestamp {
    const NAME: &'static str = "Timestamp";
    /*
     * Timestamp  =  "Timestamp" HCOLON 1*(DIGIT)
     *                [ "." *(DIGIT) ] [ LWS delay ]
     * delay      =  *(DIGIT) [ "." *(DIGIT) ]
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let time = reader.read_number_as_str();
        space!(reader);
        let has_delay = reader.peek().is_some_and(|b| !is_newline(b));

        let delay = if has_delay {
            Some(reader.read_number_as_str())
        } else {
            None
        };
        Ok(Timestamp {
            time: time.into(),
            delay: delay.map(|s| s.into()),
        })
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.time)?;

        if let Some(delay) = &self.delay {
            write!(f, "{}", delay)?;
        }

        Ok(())
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

        assert_eq!(timestamp.time, "54.0".into());
    }
}
