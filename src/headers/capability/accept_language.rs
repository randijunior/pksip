use crate::{
    headers::{self, Q_PARAM},
    macros::{parse_param, read_while, space},
    parser::{Param, Result},
    scanner::Scanner,
    uri::Params,
    util::is_alphabetic,
};

use crate::headers::SipHeaderParser;
use std::str;
#[derive(Debug, PartialEq)]
pub struct Language<'a> {
    language: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Language<'a> {
    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        space!(scanner);
        let is_lang =
            |byte: &u8| byte == &b'*' || byte == &b'-' || is_alphabetic(byte);
        let language = read_while!(scanner, is_lang);
        let language = unsafe { str::from_utf8_unchecked(language) };
        let mut q: Option<f32> = None;
        let param = parse_param!(scanner, |param: Param<'a>| {
            let (name, value) = param;
            if name == Q_PARAM {
                q = headers::parse_q_value(value);
                return None;
            }
            Some(param)
        });

        Ok(Language { language, q, param })
    }
}

#[derive(Debug, PartialEq)]
pub struct AcceptLanguage<'a>(Vec<Language<'a>>);

impl<'a> AcceptLanguage<'a> {
    pub fn get(&self, index: usize) -> Option<&Language<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for AcceptLanguage<'a> {
    const NAME: &'static [u8] = b"Accept-Language";

    fn parse(scanner: &mut Scanner<'a>) -> crate::parser::Result<Self> {
        let mut languages: Vec<Language> = Vec::new();
        space!(scanner);

        let lang = Language::parse(scanner)?;
        languages.push(lang);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            let lang = Language::parse(scanner)?;
            languages.push(lang);
            space!(scanner);
        }

        Ok(AcceptLanguage(languages))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"en\r\n";
        let mut scanner = Scanner::new(src);
        let accept_language = AcceptLanguage::parse(&mut scanner).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, None);
        assert_eq!(lang.param, None);

        let src = b"da, en-gb;q=0.8, en;q=0.7\r\n";
        let mut scanner = Scanner::new(src);
        let accept_language = AcceptLanguage::parse(&mut scanner).unwrap();

        assert!(accept_language.len() == 3);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "da");
        assert_eq!(lang.q, None);
        assert_eq!(lang.param, None);

        let lang = accept_language.get(1).unwrap();
        assert_eq!(lang.language, "en-gb");
        assert_eq!(lang.q, Some(0.8));
        assert_eq!(lang.param, None);

        let lang = accept_language.get(2).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, Some(0.7));
        assert_eq!(lang.param, None);

        let src = b"*\r\n";
        let mut scanner = Scanner::new(src);
        let accept_language = AcceptLanguage::parse(&mut scanner).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(scanner.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "*");
        assert_eq!(lang.q, None);
        assert_eq!(lang.param, None);
    }
}
