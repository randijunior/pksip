use core::str;

use crate::{macros::{read_while, space}, parser::is_token, util::is_alphabetic};

use super::SipHeaderParser;


pub struct ContentLanguage<'a>(Vec<&'a str>);


impl<'a> SipHeaderParser<'a> for ContentLanguage<'a> {
    const NAME: &'a [u8] = b"Content-Language";
    
    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        let mut languages: Vec<&'a str> = Vec::new();
        let is_lang = |byte: u8| byte == b'*' || byte == b'-' || is_alphabetic(byte);
        let language = read_while!(reader, is_lang);
        let language = unsafe { str::from_utf8_unchecked(language) };
        languages.push(language);
        
        while let Some(b',') = reader.peek() {
            reader.next();
            let language = read_while!(reader, is_token);
            let language = unsafe { str::from_utf8_unchecked(language) };
            languages.push(language);
            space!(reader);
        }

        Ok(ContentLanguage(languages))
    }

}