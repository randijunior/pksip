use crate::{
    bytes::Bytes,
    headers::{self, Q_PARAM},
    macros::{parse_comma_separated_header, parse_param},
    parser::{self, Result},
    uri::Params,
    util::is_alphabetic,
};

use crate::headers::SipHeader;
use std::str;

/// A `language` that apear in `Accept-Language` header.
pub struct Language<'a> {
    language: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

/// A `Accept-Language` SIP header.
///
/// The `Accept-Language` indicates the client's language preferences.
pub struct AcceptLanguage<'a>(Vec<Language<'a>>);

impl<'a> AcceptLanguage<'a> {
    pub fn get(&self, index: usize) -> Option<&Language<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeader<'a> for AcceptLanguage<'a> {
    const NAME: &'static str = "Accept-Language";

    fn parse(bytes: &mut Bytes<'a>) -> Result<AcceptLanguage<'a>> {
        let languages = parse_comma_separated_header!(bytes => {
            let is_lang = |byte: &u8| {
                byte == &b'*' || byte == &b'-' || is_alphabetic(byte)
            };
            let language = parser::parse_slice_utf8(bytes, is_lang);
            let mut q_param = None;
            let param = parse_param!(bytes, Q_PARAM = q_param);
            let q = q_param.and_then(|q| headers::parse_q(Some(q)));

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
        assert_eq!(lang.q, Some(0.8));

        let lang = accept_language.get(2).unwrap();
        assert_eq!(lang.language, "en");
        assert_eq!(lang.q, Some(0.7));

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
