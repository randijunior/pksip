use std::str;

use reader::{space, until, Reader};

use crate::parser::is_host;
use crate::{macros::sip_parse_error, parser::Result};

use crate::headers::SipHeader;

/// The `Warning` SIP header.
/// Carry additional information about the status of a response.
#[derive(Debug, PartialEq, Eq)]
pub struct Warning<'a> {
    code: u32,
    host: &'a str,
    text: &'a str,
}

impl<'a> SipHeader<'a> for Warning<'a> {
    const NAME: &'static str = "Warning";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let code = reader.read_num()?;
        space!(reader);
        let host = unsafe { reader.read_as_str(is_host) };
        space!(reader);
        let Some(&b'"') = reader.peek() else {
            return sip_parse_error!("invalid warning header!");
        };
        reader.next();
        let text = until!(reader, &b'"');
        reader.next();
        let text = str::from_utf8(text)?;

        Ok(Warning { code, host, text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"307 isi.edu \"Session parameter 'foo' not understood\"";
        let mut reader = Reader::new(src);
        let warn = Warning::parse(&mut reader);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host, "isi.edu");
        assert_eq!(warn.text, "Session parameter 'foo' not understood");
    }
}
