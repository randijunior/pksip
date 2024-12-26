use itertools::Itertools;
use reader::{util::is_alphabetic, Reader};

use crate::{
    headers::Q_PARAM,
    macros::{hdr_list, parse_header_param},
    msg::Params,
    parser::Result,
};

use crate::headers::SipHeader;
use std::{fmt, str};

use super::Q;

/// A `language` that apear in `Accept-Language` header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language<'a> {
    language: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
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

/// The `Accept-Language` SIP header.
///
/// Indicates the client's language preferences.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct AcceptLanguage<'a>(Vec<Language<'a>>);

impl<'a> AcceptLanguage<'a> {
    pub fn get(&self, index: usize) -> Option<&Language<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[inline]
pub(crate) fn is_lang(byte: &u8) -> bool {
    byte == &b'*' || byte == &b'-' || is_alphabetic(byte)
}

impl<'a> SipHeader<'a> for AcceptLanguage<'a> {
    const NAME: &'static str = "Accept-Language";

    fn parse(reader: &mut Reader<'a>) -> Result<AcceptLanguage<'a>> {
        let languages = hdr_list!(reader => {
            let language = unsafe { reader.read_as_str(is_lang) };
            let mut q_param = None;
            let param = parse_header_param!(reader, Q_PARAM = q_param);
            let q = q_param.and_then(|q| Q::parse(q));

            Language { language, q, param }
        });

        Ok(AcceptLanguage(languages))
    }
}

impl fmt::Display for AcceptLanguage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"en\r\n";
        let mut reader = Reader::new(src);
        let accept_language = AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(reader.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, None);

        let src = b"da, en-gb;q=0.8, en;q=0.7\r\n";
        let mut reader = Reader::new(src);
        let accept_language = AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 3);
        assert_eq!(reader.as_ref(), b"\r\n");

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
        let mut reader = Reader::new(src);
        let accept_language = AcceptLanguage::parse(&mut reader).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(reader.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "*");
        assert_eq!(lang.q, None);
    }
}
