use crate::headers::SipHeader;
use crate::macros::read_until_byte;
use crate::{
    bytes::Bytes,
    macros::{parse_param, space},
    parser::Result,
    uri::Params,
};
use core::str;

/// The `Alert-Info` SIP header.
///
/// Specifies an alternative ring tone.
pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for AlertInfo<'a> {
    const NAME: &'static str = "Alert-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<AlertInfo<'a>> {
        space!(bytes);

        bytes.must_read(b'<')?;
        let url = read_until_byte!(bytes, &b'>');
        bytes.must_read(b'>')?;

        let url = str::from_utf8(url)?;
        let params = parse_param!(bytes);

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
        let alert_info = AlertInfo::parse(&mut bytes);
        let alert_info = alert_info.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");

        let src =
            b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut bytes = Bytes::new(src);
        let alert_info = AlertInfo::parse(&mut bytes);
        let alert_info = alert_info.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(
            alert_info.params.unwrap().get("purpose"),
            Some(&"ringtone")
        );
    }
}
