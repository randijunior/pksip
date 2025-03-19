use super::{Header, ParseHeaderError};
use crate::{
    headers::{SipHeader, Q_PARAM},
    internal::{ArcStr, Q},
    macros::{hdr_list, parse_header_param},
    message::Params,
    parser::Result,
};
use itertools::Itertools;
use reader::{util::is_alphabetic, Reader};
use std::{fmt, str};

/// The `Accept-Language` SIP header.
///
/// Indicates the client's language preferences.
///
/// # Examples
///
/// ```
/// # use sip::{headers::{AcceptLanguage, accept_language::Language}};
/// let mut language = AcceptLanguage::new();
///
/// language.push(Language::new("en"));
/// language.push(Language::new("fr"));
///
/// assert_eq!("Accept-Language: en, fr".as_bytes().try_into(), Ok(language));
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptLanguage(Vec<Language>);

impl AcceptLanguage {
    /// Creates a empty `AcceptLanguage` header.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an new `Language` at the end of the header.
    #[inline]
    pub fn push(&mut self, lang: Language) {
        self.0.push(lang);
    }

    /// Returns a reference to an `Language` at the specified index.
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

impl TryFrom<&[u8]> for AcceptLanguage {
    type Error = ParseHeaderError;

    fn try_from(
        value: &[u8],
    ) -> std::result::Result<Self, Self::Error> {
        Header::from_bytes(value)?
            .into_accept_language()
            .map_err(|_| ParseHeaderError(Self::NAME))
    }
}

#[inline]
pub(crate) fn is_lang(byte: &u8) -> bool {
    byte == &b'*' || byte == &b'-' || is_alphabetic(byte)
}

impl SipHeader<'_> for AcceptLanguage {
    const NAME: &'static str = "Accept-Language";
    /*
     * Accept-Language  =  "Accept-Language" HCOLON
     *                      [ language *(COMMA language) ]
     * language         =  language-range *(SEMI accept-param)
     * language-range   =  ( ( 1*8ALPHA *( "-" 1*8ALPHA ) ) / "*" )
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let languages = hdr_list!(reader => {
            let language = unsafe { reader.read_as_str(is_lang) };
            let mut q_param = None;
            let param = parse_header_param!(reader, Q_PARAM = q_param);
            let q = q_param.map(|q| q.parse()).transpose()?;

            Language { language: language.into(), q, param }
        });

        Ok(AcceptLanguage(languages))
    }
}

impl fmt::Display for AcceptLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

/// A `language` that apear in `Accept-Language` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    language: ArcStr,
    q: Option<Q>,
    param: Option<Params>,
}

impl Language {
    /// Creates a new `Language` instance.
    pub fn new(language: &str) -> Self {
        Self {
            language: language.into(),
            q: None,
            param: None,
        }
    }

    pub fn from_parts(
        language: &str,
        q: Option<Q>,
        param: Option<Params>,
    ) -> Self {
        Self {
            language: language.into(),
            q,
            param,
        }
    }
}

impl fmt::Display for Language {
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
        let mut reader = Reader::new(src);
        let accept_language =
            AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(reader.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "en".into());
        assert_eq!(lang.q, None);

        let src = b"da, en-gb;q=0.8, en;q=0.7\r\n";
        let mut reader = Reader::new(src);
        let accept_language =
            AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "da".into());
        assert_eq!(lang.q, None);

        let lang = accept_language.get(1).unwrap();
        assert_eq!(lang.language, "en-gb".into());
        assert_eq!(lang.q, Some(Q(0, 8)));

        let lang = accept_language.get(2).unwrap();
        assert_eq!(lang.language, "en".into());
        assert_eq!(lang.q, Some(Q(0, 7)));

        let src = b"*\r\n";
        let mut reader = Reader::new(src);
        let accept_language =
            AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(reader.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "*".into());
        assert_eq!(lang.q, None);
    }
}
