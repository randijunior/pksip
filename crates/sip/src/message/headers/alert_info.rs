use crate::{
    bytes::Bytes,
    macros::{parse_header_param, read_while, sip_parse_error, space},
    parser::Result,
    uri::Params,
    util::is_newline,
};

pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

use crate::headers::SipHeaderParser;

use std::str;

impl<'a> SipHeaderParser<'a> for AlertInfo<'a> {
    const NAME: &'static [u8] = b"Alert-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        space!(bytes);
        // must be an '<'
        let Some(&b'<') = bytes.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let url =
            read_while!(bytes, |b| !matches!(b, b'>' | b';') && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = bytes.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let params = parse_header_param!(bytes);

        Ok(AlertInfo { url, params })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut bytes = Bytes::new(src);
        let alert_info = AlertInfo::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");

        let src =
            b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut bytes = Bytes::new(src);
        let alert_info = AlertInfo::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(
            alert_info.params.unwrap().get("purpose"),
            Some(&"ringtone")
        );
    }
}
