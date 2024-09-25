use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::Result,
    uri::Params,
    util::is_newline,
};

pub struct AlertInfo<'a> {
    url: &'a str,
    params: Option<Params<'a>>,
}

use super::SipHeaderParser;

use std::str;

impl<'a> SipHeaderParser<'a> for AlertInfo<'a> {
    const NAME: &'static [u8] = b"Alert-Info";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        space!(reader);
        // must be an '<'
        let Some(&b'<') = reader.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let url = read_while!(reader, |b| !matches!(b, b'>' | b';') && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = reader.next() else {
            return sip_parse_error!("Invalid alert info!");
        };
        let params = parse_param!(reader, |param| Some(param));

        Ok(AlertInfo { url, params })
    }
}
