use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, sip_parse_error},
    parser::{Param, Result},
    uri::Params, util::is_newline,
};

use super::SipHeaderParser;

use std::str;

/*
Call-Info   =  "Call-Info" HCOLON info *(COMMA info)
info        =  LAQUOT absoluteURI RAQUOT *( SEMI info-param)
info-param  =  ( "purpose" EQUAL ( "icon" / "info"
               / "card" / token ) ) / generic-param
*/
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for CallInfo<'a> {
    const NAME: &'a [u8] = b"Call-Info";
    
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        let Some(&b'<') = reader.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        let url = read_while!(reader, |b| !matches!(b, b'>' | b';') && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = reader.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        let params = parse_param!(reader, |param: Param<'a>| {
            let (name, value) = param;
            if name == "purpose" {
               purpose = value;
               return None;
            }
            Some(param)
        });


        Ok(CallInfo { url, params, purpose })
    }
}