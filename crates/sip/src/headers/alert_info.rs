use scanner::{space, until_byte, Scanner};

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
pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for AlertInfo<'a> {
    const NAME: &'static str = "Alert-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<AlertInfo<'a>> {
        space!(scanner);

        scanner.must_read(b'<')?;
        let url = until_byte!(scanner, &b'>');
        scanner.must_read(b'>')?;

        let url = str::from_utf8(url)?;
        let params = parse_header_param!(scanner);

        Ok(AlertInfo { url, params })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut scanner = Scanner::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");

        let src =
            b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut scanner = Scanner::new(src);
        let alert_info = AlertInfo::parse(&mut scanner);
        let alert_info = alert_info.unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(
            alert_info.params.unwrap().get("purpose"),
            Some(&"ringtone")
        );
    }
}
