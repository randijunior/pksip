use std::{fmt, str};

use reader::{space, until, Reader};

use crate::internal::ArcStr;
use crate::parser::is_host;
use crate::{macros::sip_parse_error, parser::Result};

use crate::headers::SipHeader;

/// The `Warning` SIP header.
/// Carry additional information about the status of a response.
#[derive(Debug, PartialEq, Eq)]
pub struct Warning {
    code: u32,
    host: ArcStr,
    text: ArcStr,
}

impl SipHeader<'_> for Warning {
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
    fn parse(reader: &mut Reader) -> Result<Self> {
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

        Ok(Warning {
            code,
            host: host.into(),
            text: text.into(),
        })
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.code, self.host, self.text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src =
            b"307 isi.edu \"Session parameter 'foo' not understood\"";
        let mut reader = Reader::new(src);
        let warn = Warning::parse(&mut reader);
        let warn = warn.unwrap();

        assert_eq!(warn.code, 307);
        assert_eq!(warn.host, "isi.edu".into());
        assert_eq!(
            warn.text,
            "Session parameter 'foo' not understood".into()
        );
    }
}
