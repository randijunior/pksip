use core::str;

use crate::{
    bytes::Bytes,
    macros::{read_while, space},
    parser::{is_token, Result},
    util::is_alphabetic,
};

use crate::headers::SipHeader;


/// Specifies the language of the `message-body` content.
pub struct ContentLanguage<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ContentLanguage<'a> {
    const NAME: &'static str = "Content-Language";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut languages: Vec<&'a str> = Vec::new();
        let is_lang =
            |byte: &u8| byte == &b'*' || byte == &b'-' || is_alphabetic(byte);
        let language = read_while!(bytes, is_lang);
        let language = unsafe { str::from_utf8_unchecked(language) };
        languages.push(language);

        space!(bytes);
        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let language = read_while!(bytes, is_token);
            let language = unsafe { str::from_utf8_unchecked(language) };
            languages.push(language);
        }

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
        let c_lang = ContentLanguage::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_lang.0.get(0), Some(&"fr"));

        let src = b"fr, en\r\n";
        let mut bytes = Bytes::new(src);
        let c_lang = ContentLanguage::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");

        assert_eq!(c_lang.0.get(0), Some(&"fr"));
        assert_eq!(c_lang.0.get(1), Some(&"en"));
    }
}
