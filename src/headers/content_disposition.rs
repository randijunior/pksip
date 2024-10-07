use crate::{
    scanner::Scanner,
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

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let disp_type = read_while!(scanner, is_token);
        let disp_type = unsafe { std::str::from_utf8_unchecked(disp_type) };
        space!(scanner);
        let params = parse_param!(scanner, |param| Some(param));

        Ok(ContentDisposition { disp_type, params })
    }
}
