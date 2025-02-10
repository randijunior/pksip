use std::{fmt, str};

use itertools::Itertools;
use reader::Reader;

use crate::headers::accept_language::is_lang;
use crate::internal::ArcStr;
use crate::{macros::hdr_list, parser::Result};

use crate::headers::SipHeader;

/// The `Content-Language` SIP header.
///
/// Specifies the language of the `message-body` content.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentLanguage(Vec<ArcStr>);

impl SipHeader<'_> for ContentLanguage {
    const NAME: &'static str = "Content-Language";
    /*
     * Content-Language  =  "Content-Language" HCOLON
     *                      language-tag *(COMMA language-tag)
     * language-tag      =  primary-tag *( "-" subtag )
     * primary-tag       =  1*8ALPHA
     * subtag            =  1*8ALPHA
     */
    fn parse(reader: &mut Reader) -> Result<ContentLanguage> {
        let languages = hdr_list!(reader => unsafe {
            reader.read_as_str(is_lang).into()
        });

        Ok(ContentLanguage(languages))
    }
}

impl fmt::Display for ContentLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"fr\r\n";
        let mut reader = Reader::new(src);
        let lang = ContentLanguage::parse(&mut reader);
        let lang = lang.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(lang.0.get(0), Some(&"fr".into()));

        let src = b"fr, en\r\n";
        let mut reader = Reader::new(src);
        let lang = ContentLanguage::parse(&mut reader);
        let lang = lang.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_eq!(lang.0.get(0), Some(&"fr".into()));
        assert_eq!(lang.0.get(1), Some(&"en".into()));
    }
}
