use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, space},
    parser::{is_token, Result},
    uri::Params,
};

use super::SipHeaderParser;

pub struct ContentDisposition<'a> {
    disp_type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ContentDisposition<'a> {
    const NAME: &'static [u8] = b"Content-Disposition";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let disp_type = read_while!(reader, is_token);
        let disp_type = unsafe { std::str::from_utf8_unchecked(disp_type) };
        space!(reader);
        let params = parse_param!(reader, |param| Some(param));

        Ok(ContentDisposition { disp_type, params })
    }
}
