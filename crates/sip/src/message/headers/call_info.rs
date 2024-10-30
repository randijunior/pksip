use crate::{
    bytes::Bytes,
    macros::{parse_header_param, read_while, sip_parse_error, space},
    parser::Result,
    uri::Params,
    util::is_newline,
};

use crate::headers::SipHeaderParser;

use std::str;
const PURPOSE: &'static str = "purpose";

/// Provides aditional information aboute the caller or calle.
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for CallInfo<'a> {
    const NAME: &'static str = "Call-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        let Some(&b'<') = bytes.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        let url =
            read_while!(bytes, |b| !matches!(b, b'>' | b';') && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = bytes.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        space!(bytes);
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
