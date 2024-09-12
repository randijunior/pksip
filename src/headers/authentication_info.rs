use crate::{
    byte_reader::ByteReader,
    macros::{parse_auth_param, read_while, sip_parse_error, space},
    parser::{is_token, Result},
    uri::Params,
};

use super::SipHeaderParser;

use std::str;

pub struct AuthenticationInfo<'a> {
    nextnonce: Option<&'a str>,
    qop: Option<&'a str>,
    rspauth: Option<&'a str>,
    cnonce: Option<&'a str>,
    nc: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for AuthenticationInfo<'a> {
    const NAME: &'a [u8] = b"Authentication-Info";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut nextnonce: Option<&'a str> = None;
        let mut rspauth: Option<&'a str> = None;
        let mut qop: Option<&'a str> = None;
        let mut cnonce: Option<&'a str> = None;
        let mut nc: Option<&'a str> = None;

        macro_rules! parse {
            () => {
                space!(reader);
                match read_while!(reader, is_token) {
                    b"nextnonce" => nextnonce = parse_auth_param!(reader),
                    b"qop" => qop = parse_auth_param!(reader),
                    b"rspauth" => rspauth = parse_auth_param!(reader),
                    b"cnonce" => cnonce = parse_auth_param!(reader),
                    b"nc" => nc = parse_auth_param!(reader),
                    _ => sip_parse_error!("Can't parse Authentication-Info")?,
                };
            };
        }

        parse!();
        while let Some(b',') = reader.peek() {
            reader.next();
            parse!();
        }

        Ok(AuthenticationInfo {
            nextnonce,
            qop,
            rspauth,
            cnonce,
            nc,
        })
    }
}
