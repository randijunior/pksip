use crate::{
    bytes::Bytes,
    headers::{self, Q_PARAM},
    macros::{parse_header_list, parse_header_param, parse_param},
    parser::{self, Result},
    token::Token,
    uri::Params,
    util::is_alphabetic,
};

use crate::headers::SipHeader;
use std::str;

use super::Q;

/// A `language` that apear in `Accept-Language` header.
#[derive(Debug, Clone)]
pub struct Language<'a> {
    language: &'a str,
    q: Option<Q>,
    param: Option<Params<'a>>,
}

/// The `Accept-Language` SIP header.
///
/// Indicates the client's language preferences.
#[derive(Default, Debug, Clone)]
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

    fn parse(bytes: &mut Bytes<'a>) -> Result<AcceptLanguage<'a>> {
        let languages = parse_header_list!(bytes => {
            let language = unsafe { bytes.parse_str(is_lang) };
            let mut q_param = None;
            let param = parse_header_param!(bytes, Q_PARAM = q_param);
            let q = q_param.and_then(|q| headers::parse_q(q));

            Language { language, q, param }
        });

        Ok(AcceptLanguage(languages))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"en\r\n";
        let mut bytes = Bytes::new(src);
        let accept_language = AcceptLanguage::parse(&mut bytes).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(bytes.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, None);

        let src = b"da, en-gb;q=0.8, en;q=0.7\r\n";
        let mut bytes = Bytes::new(src);
        let accept_language = AcceptLanguage::parse(&mut bytes).unwrap();

        assert!(accept_language.len() == 3);
        assert_eq!(bytes.as_ref(), b"\r\n");

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
        let mut bytes = Bytes::new(src);
        let accept_language = AcceptLanguage::parse(&mut bytes).unwrap();

        assert!(accept_language.len() == 1);
        assert_eq!(bytes.as_ref(), b"\r\n");

        let lang = accept_language.get(0).unwrap();
        assert_eq!(lang.language, "*");
        assert_eq!(lang.q, None);
    }
}
