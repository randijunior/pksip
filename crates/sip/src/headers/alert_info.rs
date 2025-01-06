use reader::{space, until, Reader};

use crate::headers::SipHeader;
use crate::{macros::parse_header_param, message::Params, parser::Result};
use std::{fmt, str};

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
    /*
     * Alert-Info   =  "Alert-Info" HCOLON alert-param *(COMMA alert-param)
     * alert-param  =  LAQUOT absoluteURI RAQUOT *( SEMI generic-param )
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        space!(reader);

        reader.must_read(b'<')?;
        let url = until!(reader, &b'>');
        reader.must_read(b'>')?;

        let url = str::from_utf8(url)?;
        let params = parse_header_param!(reader);

        Ok(AlertInfo { url, params })
    }
}

impl fmt::Display for AlertInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.url)?;
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
