use std::str;

use reader::Reader;

use crate::headers::accept_language::is_lang;
use crate::{macros::parse_header_list, parser::Result};

use crate::headers::SipHeader;

/// The `Content-Language` SIP header.
///
/// Specifies the language of the `message-body` content.
#[derive(Debug, PartialEq, Eq)]
pub struct ContentLanguage<'a>(Vec<&'a str>);

impl<'a> SipHeader<'a> for ContentLanguage<'a> {
    const NAME: &'static str = "Content-Language";

    fn parse(reader: &mut Reader<'a>) -> Result<ContentLanguage<'a>> {
        let languages = parse_header_list!(reader => unsafe {
            reader.read_while_as_str(is_lang)
        });

        Ok(ContentLanguage(languages))
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
        assert_eq!(lang.0.get(0), Some(&"fr"));

        let src = b"fr, en\r\n";
        let mut reader = Reader::new(src);
        let lang = ContentLanguage::parse(&mut reader);
        let lang = lang.unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_eq!(lang.0.get(0), Some(&"fr"));
        assert_eq!(lang.0.get(1), Some(&"en"));
    }
}
