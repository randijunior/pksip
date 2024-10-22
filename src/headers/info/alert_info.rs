use crate::{
    macros::{parse_header_param, read_while, sip_parse_error, space},
    parser::Result,
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

#[derive(Debug, PartialEq, Eq)]
pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

use crate::headers::SipHeaderParser;

use std::str;

impl<'a> SipHeaderParser<'a> for AlertInfo<'a> {
    const NAME: &'static [u8] = b"Alert-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        space!(scanner);
        // must be an '<'
        let Some(&b'<') = scanner.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let url = read_while!(scanner, |b| !matches!(b, b'>' | b';')
            && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = scanner.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let params = parse_header_param!(scanner);

        Ok(AlertInfo { url, params })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://www.example.com/sounds/moo.wav>\r\n";
        let mut scanner = Scanner::new(src);
        let alert_info = AlertInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://www.example.com/sounds/moo.wav");
        assert_eq!(alert_info.params, None);

        let src =
            b"<http://example.com/ringtones/premium.wav>;purpose=ringtone\r\n";
        let mut scanner = Scanner::new(src);
        let alert_info = AlertInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(alert_info.url, "http://example.com/ringtones/premium.wav");
        assert_eq!(
            alert_info.params,
            Some(Params::from(HashMap::from([("purpose", Some("ringtone"))])))
        );
    }
}
