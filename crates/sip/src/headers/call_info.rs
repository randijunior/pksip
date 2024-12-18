use reader::{until, Reader};

use crate::{macros::parse_header_param, msg::Params, parser::Result};

use crate::headers::SipHeader;

use std::{fmt, str};
const PURPOSE: &'static str = "purpose";

/// The `Call-Info` SIP header.
///
/// Provides aditional information aboute the caller or calle.
#[derive(Debug, PartialEq, Eq)]
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for CallInfo<'a> {
    const NAME: &'static str = "Call-Info";

    fn parse(reader: &mut Reader<'a>) -> Result<CallInfo<'a>> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        reader.must_read(b'<')?;
        let url = until!(reader, &b'>');
        // must be an '>'
        reader.must_read(b'>')?;
        let url = str::from_utf8(url)?;
        let params = parse_header_param!(reader, PURPOSE = purpose);

        Ok(CallInfo {
            url,
            params,
            purpose,
        })
    }
}

impl fmt::Display for CallInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.url)?;
        if let Some(purpose) = self.purpose {
            write!(f, ";{}", purpose)?;
        }
        if let Some(params) = &self.params {
            write!(f, ";{}", params)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> \
        ;purpose=icon\r\n";
        let mut reader = Reader::new(src);
        let info = CallInfo::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon"));

        let src = b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut reader = Reader::new(src);
        let info = CallInfo::parse(&mut reader).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info"));
    }
}
