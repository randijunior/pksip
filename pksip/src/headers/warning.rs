use std::{fmt, str};

use crate::parser::{is_host, ParseCtx};
use crate::{error::Result, macros::parse_error};

use crate::headers::SipHeaderParse;

/// The `Warning` SIP header.
/// Carry additional information about the status of a
/// response.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Warning<'a> {
    code: u32,
    host: &'a str,
    text: &'a str,
}

impl<'a> SipHeaderParse<'a> for Warning<'a> {
    const NAME: &'static str = "Warning";
    /*
     * Warning        =  "Warning" HCOLON warning-value *(COMMA warning-value)
     * warning-value  =  warn-code SP warn-agent SP warn-text
     * warn-code      =  3DIGIT
     * warn-agent     =  hostport / pseudonym
     *                   ;  the name or pseudonym of the server adding
     *                   ;  the Warning header, for use in debugging
     * warn-text      =  quoted-string
     * pseudonym      =  token
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let code = parser.parse_u32()?;
        parser.take_ws();
        let host = unsafe { parser.read_as_str(is_host) };
        parser.take_ws();
        let Some(b'"') = parser.peek() else {
            return parse_error!("invalid warning header!");
        };
        parser.advance();
        let text = parser.read_until_byte(b'"');
        parser.advance();
        let text = str::from_utf8(text)?;

        Ok(Warning { code, host, text })
    }
}

impl fmt::Display for Warning<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {} {}", Warning::NAME, self.code, self.host, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"307 isi.edu \"Session parameter 'foo' not understood\"";
        let mut scanner = ParseCtx::new(src);
        let warn = Warning::parse(&mut scanner);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host, "isi.edu");
        assert_eq!(warn.text, "Session parameter 'foo' not understood");
    }
}
