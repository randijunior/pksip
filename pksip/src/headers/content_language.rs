use std::{fmt, str};

use itertools::Itertools;

use crate::headers::accept_language::is_lang;

use crate::parser::Parser;
use crate::{error::Result, macros::hdr_list};

use crate::headers::SipHeaderParse;

/// The `Content-Language` SIP header.
///
/// Specifies the language of the `message-body` content.
///
/// # Examples
///
/// ```
/// # use pksip::headers::ContentLanguage;
/// let c_language = ContentLanguage::from(["fr", "en"]);
///
/// assert_eq!(
///     "Content-Language: fr, en",
///     c_language.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ContentLanguage<'a>(Vec<&'a str>);

impl<'a> SipHeaderParse<'a> for ContentLanguage<'a> {
    const NAME: &'static str = "Content-Language";
    /*
     * Content-Language  =  "Content-Language" HCOLON
     *                      language-tag *(COMMA language-tag)
     * language-tag      =  primary-tag *( "-" subtag )
     * primary-tag       =  1*8ALPHA
     * subtag            =  1*8ALPHA
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let languages = hdr_list!(parser => unsafe {
            parser.read_as_str(is_lang)
        });

        Ok(ContentLanguage(languages))
    }
}

impl fmt::Display for ContentLanguage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ContentLanguage::NAME, self.0.iter().format(", "))
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for ContentLanguage<'a> {
    fn from(value: [&'a str; N]) -> Self {
        Self(Vec::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"fr\r\n";
        let mut scanner = Parser::new(src);
        let lang = ContentLanguage::parse(&mut scanner);
        let lang = lang.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");
        assert_eq!(lang.0.get(0), Some(&"fr".into()));

        let src = b"fr, en\r\n";
        let mut scanner = Parser::new(src);
        let lang = ContentLanguage::parse(&mut scanner);
        let lang = lang.unwrap();

        assert_eq!(scanner.remaing(), b"\r\n");

        assert_eq!(lang.0.get(0), Some(&"fr".into()));
        assert_eq!(lang.0.get(1), Some(&"en".into()));
    }
}
