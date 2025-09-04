use std::fmt;
use std::str;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::macros::parse_error;
use crate::parser::is_host;
use crate::parser::Parser;
use crate::ArcStr;

/// The `Warning` SIP header.
/// Carry additional information about the status of a
/// response.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Warning {
    code: u32,
    host: ArcStr,
    text: ArcStr,
}

impl<'a> HeaderParser<'a> for Warning {
    const NAME: &'static str = "Warning";

    /*
     * Warning        =  "Warning" HCOLON warning-value
     * *(COMMA warning-value) warning-value  =  warn-code
     * SP warn-agent SP warn-text warn-code      =
     * 3DIGIT warn-agent     =  hostport / pseudonym
     *                   ;  the name or pseudonym of the
     * server adding                   ;  the Warning
     * header, for use in debugging warn-text      =
     * quoted-string pseudonym      =  token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let code = parser.parse_u32()?;
        parser.space();
        let host = unsafe { parser.read_while_as_str_unchecked(is_host) };
        parser.space();
        let Some(b'"') = parser.peek_byte() else {
            return parse_error!("invalid warning header!".into());
        };
        parser.next_byte();
        let text = parser.read_until_byte(b'"');
        parser.next_byte();
        let text = str::from_utf8(text)?;

        Ok(Warning {
            code,
            host: host.into(),
            text: text.into(),
        })
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {} {} {}",
            Warning::NAME,
            self.code,
            self.host,
            self.text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"307 isi.edu \"InvSession parameter 'foo' not understood\"";
        let mut scanner = Parser::new(src);
        let warn = Warning::parse(&mut scanner);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host.as_ref(), "isi.edu");
        assert_eq!(
            warn.text.as_ref(),
            "InvSession parameter 'foo' not understood"
        );
    }
}
