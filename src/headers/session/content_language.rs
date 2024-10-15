use core::str;

use crate::{
    macros::{read_while, space}, parser::{is_token, Result}, scanner::Scanner, util::is_alphabetic
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct ContentLanguage<'a>(Vec<&'a str>);

impl<'a> SipHeaderParser<'a> for ContentLanguage<'a> {
    const NAME: &'static [u8] = b"Content-Language";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut languages: Vec<&'a str> = Vec::new();
        let is_lang = |byte: u8| byte == b'*' || byte == b'-' || is_alphabetic(byte);
        let language = read_while!(scanner, is_lang);
        let language = unsafe { str::from_utf8_unchecked(language) };
        languages.push(language);

        space!(scanner);
        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);
            let language = read_while!(scanner, is_token);
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
        let mut scanner = Scanner::new(src);
        let c_lang = ContentLanguage::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_lang, ContentLanguage(vec!["fr"]));

        let src = b"fr, en\r\n";
        let mut scanner = Scanner::new(src);
        let c_lang = ContentLanguage::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_lang, ContentLanguage(vec!["fr", "en"]));

    }
}

