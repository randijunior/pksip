use std::str;

use crate::{
    bytes::Bytes,
    macros::{until_byte, sip_parse_error, space},
    parser::Result,
    uri::is_host,
};

use crate::headers::SipHeader;

/// The `Warning` SIP header.
/// Carry additional information about the status of a response.
pub struct Warning<'a> {
    code: u32,
    host: &'a str,
    text: &'a str,
}

impl<'a> SipHeader<'a> for Warning<'a> {
    const NAME: &'static str = "Warning";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let code = bytes.parse_num()?;
        space!(bytes);
        let host = unsafe { bytes.parse_str(is_host) };
        space!(bytes);
        let Some(&b'"') = bytes.peek() else {
            return sip_parse_error!("invalid warning header!");
        };
        bytes.next();
        let text = until_byte!(bytes, &b'"');
        bytes.next();
        let text = str::from_utf8(text)?;

        Ok(Warning { code, host, text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src =
            b"307 isi.edu \"Session parameter 'foo' not understood\"";
        let mut bytes = Bytes::new(src);
        let warn = Warning::parse(&mut bytes);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host, "isi.edu");
        assert_eq!(warn.text, "Session parameter 'foo' not understood");
    }
}
