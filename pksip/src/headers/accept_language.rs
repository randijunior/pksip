use crate::{
    error::Result,
    headers::{SipHeaderParse, Q_PARAM},
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::ParseCtx,
    Q,
};
use itertools::Itertools;
use std::{fmt, str};
use pksip_util::util::is_alphabetic;

/// The `Accept-Language` SIP header.
///
/// Indicates the client's language preferences.
///
/// # Examples
///
/// ```
/// # use pksip::{headers::{AcceptLanguage, Language}};
/// let mut language = AcceptLanguage::new();
///
/// language.push(Language::new("en"));
/// language.push(Language::new("fr"));
///
/// assert_eq!(
///     language.to_string(),
///     "Accept-Language: en, fr",
/// );
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptLanguage<'a>(Vec<Language<'a>>);

impl<'a> AcceptLanguage<'a> {
    /// Creates a empty `AcceptLanguage` header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an new `Language` at the end of the header.
    #[inline]
    pub fn push(&mut self, lang: Language<'a>) {
        self.0.push(lang);
    }

    /// Returns a reference to an `Language` at the
    /// specified index.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Language> {
        self.0.get(index)
    }

    /// Returns the number of `Languages` in the header.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[inline]
pub(crate) fn is_lang(byte: u8) -> bool {
    byte == b'*' || byte == b'-' || is_alphabetic(byte)
}

impl<'a> SipHeaderParse<'a> for AcceptLanguage<'a> {
    const NAME: &'static str = "Accept-Language";
    /*
     * Accept-Language  =  "Accept-Language" HCOLON
     *                      [ language *(COMMA language) ]
     * language         =  language-range *(SEMI accept-param)
     * language-range   =  ( ( 1*8ALPHA *( "-" 1*8ALPHA ) ) / "*" )
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let languages = hdr_list!(parser => {
            let language = unsafe { parser.read_as_str(is_lang) };
            let mut q_param = None;
            let param = parse_header_param!(parser, Q_PARAM = q_param);
            let q = q_param.map(|q| q.parse()).transpose()?;

            Language { language, q, param }
        });

        Ok(AcceptLanguage(languages))
    }
}

impl fmt::Display for AcceptLanguage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", AcceptLanguage::NAME, self.0.iter().format(", "))
    }
}

/// A `language` that apear in `Accept-Language` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language<'a> {
    language: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
}

impl<'a> Language<'a> {
    /// Creates a new `Language` instance.
    pub fn new(language: &'a str) -> Self {
        Self {
            language,
            q: None,
            param: None,
        }
    }
    /// Creates a new `Language` header with the given language, q param and another params.
    pub fn from_parts(language: &'a str, q: Option<Q>, param: Option<Params<'a>>) -> Self {
        Self { language, q, param }
    }
}

impl fmt::Display for Language<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Language { language, q, param } = self;
        write!(f, "{}", language)?;
        if let Some(q) = q {
            write!(f, "{}", q)?;
        }
        if let Some(param) = param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"en\r\n";
        let mut parser = ParseCtx::new(src);
        let accept_language = AcceptLanguage::parse(&mut parser).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(parser.remaing(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, None);

        let src = b"da, en-gb;q=0.8, en;q=0.7\r\n";
        let mut parser = ParseCtx::new(src);
        let accept_language = AcceptLanguage::parse(&mut parser).unwrap();

        assert!(accept_language.len() == 3);
        assert_eq!(parser.remaing(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "da");
        assert_eq!(lang.q, None);

        let lang = accept_language.get(1).unwrap();
        assert_eq!(lang.language, "en-gb");
        assert_eq!(lang.q, Some(Q(0, 8)));

        let lang = accept_language.get(2).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, Some(Q(0, 7)));

        let src = b"*\r\n";
        let mut parser = ParseCtx::new(src);
        let accept_language = AcceptLanguage::parse(&mut parser).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(parser.remaing(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "*");
        assert_eq!(lang.q, None);
    }
}
