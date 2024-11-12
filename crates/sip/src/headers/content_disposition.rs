use crate::{
    bytes::Bytes, macros::parse_header_param, parser::Result, token::Token,
    uri::Params,
};

use crate::headers::SipHeader;

/// The `Content-Disposition` SIP header.
///
/// Describes how the `message-body` is to be interpreted by the `UAC` or `UAS`.
pub struct ContentDisposition<'a> {
    _type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for ContentDisposition<'a> {
    const NAME: &'static str = "Content-Disposition";

    fn parse(bytes: &mut Bytes<'a>) -> Result<ContentDisposition<'a>> {
        let _type = Token::parse(bytes);
        let params = parse_header_param!(bytes);

        Ok(ContentDisposition { _type, params })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"session\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");

        let src = b"session;handling=optional\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "session");
        assert_eq!(disp.params.unwrap().get("handling"), Some(&"optional"));

        let src = b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes);
        let disp = disp.unwrap();
        assert_eq!(disp._type, "attachment");
        let params = disp.params.unwrap();

        assert_eq!(params.get("filename"), Some(&"smime.p7s"));
        assert_eq!(params.get("handling"), Some(&"required"));
    }
}
