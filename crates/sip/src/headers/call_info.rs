use crate::{
    bytes::Bytes,
    macros::{parse_header_param, until_byte},
    parser::Result,
    uri::Params,
};

use crate::headers::SipHeader;

use std::str;
const PURPOSE: &'static str = "purpose";

/// The `Call-Info` SIP header.
///
/// Provides aditional information aboute the caller or calle.
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for CallInfo<'a> {
    const NAME: &'static str = "Call-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<CallInfo<'a>> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        bytes.must_read(b'<')?;
        let url = until_byte!(bytes, &b'>');
        // must be an '>'
        bytes.must_read(b'>')?;
        let url = str::from_utf8(url)?;
        let params = parse_header_param!(bytes, PURPOSE = purpose);

        Ok(CallInfo {
            url,
            params,
            purpose,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> \
        ;purpose=icon\r\n";
        let mut bytes = Bytes::new(src);
        let info = CallInfo::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon"));

        let src = b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut bytes = Bytes::new(src);
        let info = CallInfo::parse(&mut bytes).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info"));
    }
}
