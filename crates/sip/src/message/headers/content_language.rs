use core::str;

use crate::headers::accept_language::is_lang;
use crate::parser;
use crate::{
    bytes::Bytes, macros::parse_header_list, parser::Result, token::Token,
};

use crate::headers::SipHeader;

/// The `Content-Language` SIP header.
///
/// Specifies the language of the `message-body` content.
pub struct ContentLanguage<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ContentLanguage<'a> {
    const NAME: &'static str = "Content-Language";

    fn parse(bytes: &mut Bytes<'a>) -> Result<ContentLanguage<'a>> {
        let languages = parse_header_list!(bytes => unsafe {
            bytes.parse_str(is_lang)
        });

        Ok(ContentLanguage(languages))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"fr\r\n";
        let mut bytes = Bytes::new(src);
        let lang = ContentLanguage::parse(&mut bytes);
        let lang = lang.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(lang.0.get(0), Some(&"fr"));

        let src = b"fr, en\r\n";
        let mut bytes = Bytes::new(src);
        let lang = ContentLanguage::parse(&mut bytes);
        let lang = lang.unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");

        assert_eq!(lang.0.get(0), Some(&"fr"));
        assert_eq!(lang.0.get(1), Some(&"en"));
    }
}
