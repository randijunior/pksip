use std::str;

use scanner::{space, until_byte, Scanner};

use crate::{
    macros::sip_parse_error,
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

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let code = scanner.read_num()?;
        space!(scanner);
        let host = unsafe { scanner.read_and_convert_to_str(is_host) };
        space!(scanner);
        let Some(&b'"') = scanner.peek() else {
            return sip_parse_error!("invalid warning header!");
        };
        scanner.next();
        let text = until_byte!(scanner, &b'"');
        scanner.next();
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
        let mut scanner = Scanner::new(src);
        let warn = Warning::parse(&mut scanner);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host, "isi.edu");
        assert_eq!(warn.text, "Session parameter 'foo' not understood");
    }
}
