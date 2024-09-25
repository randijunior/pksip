use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, space},
    parser::{Param, Result, Q_PARAM},
    uri::Params,
    util::is_alphabetic,
};

use super::SipHeaderParser;
use std::str;

pub struct Language<'a> {
    language: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Language<'a> {
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let is_lang = |byte: u8| byte == b'*' || byte == b'-' || is_alphabetic(byte);
        let language = read_while!(reader, is_lang);
        let language = unsafe { str::from_utf8_unchecked(language) };
        let mut q: Option<f32> = None;
        let param = parse_param!(reader, |param: Param<'a>| {
            let (name, value) = param;
            if name == Q_PARAM {
                q = AcceptLanguage::parse_q_value(value);
                return None;
            }
            Some(param)
        });

        Ok(Language { language, q, param })
    }
}
pub struct AcceptLanguage<'a> {
    languages: Vec<Language<'a>>,
}

impl<'a> SipHeaderParser<'a> for AcceptLanguage<'a> {
    const NAME: &'static [u8] = b"Accept-Language";

    fn parse(reader: &mut ByteReader<'a>) -> crate::parser::Result<Self> {
        let mut languages: Vec<Language> = Vec::new();
        space!(reader);

        let lang = Language::parse(reader)?;
        languages.push(lang);

        while let Some(b',') = reader.peek() {
            reader.next();
            let lang = Language::parse(reader)?;
            languages.push(lang);
            space!(reader);
        }

        Ok(AcceptLanguage { languages })
    }
}
