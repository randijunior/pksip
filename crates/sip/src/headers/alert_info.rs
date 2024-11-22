use reader::{space, until_byte, Reader};

use crate::headers::SipHeader;
use crate::{
    macros::parse_header_param,
    parser::Result,
    uri::Params,
};
use std::str;

/// The `Alert-Info` SIP header.
///
/// Specifies an alternative ring tone.
#[derive(Debug, PartialEq, Eq)]
pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for AlertInfo<'a> {
    const NAME: &'static str = "Alert-Info";

    fn parse(reader: &mut Reader<'a>) -> Result<AlertInfo<'a>> {
        space!(reader);

        reader.must_read(b'<')?;
        let url = until_byte!(reader, &b'>');
        reader.must_read(b'>')?;

        let url = str::from_utf8(url)?;
        let params = parse_header_param!(reader);

        Ok(AlertInfo { url, params })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut reader = Reader::new(src);
        let alert_info = AlertInfo::parse(&mut reader);
        let alert_info = alert_info.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");

        let src =
            b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut reader = Reader::new(src);
        let alert_info = AlertInfo::parse(&mut reader);
        let alert_info = alert_info.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(
            alert_info.params.unwrap().get("purpose"),
            Some(&"ringtone")
        );
    }
}
