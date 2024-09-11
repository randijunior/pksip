use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, sip_parse_error},
    parser::{Result, SipParser},
    uri::{NameAddr, Params, SipUri},
};

use super::SipHeaderParser;

use std::str;

pub struct Route<'a> {
    pub(crate) name_addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for Route<'a> {
    const NAME: &'a [u8] = b"Route";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(reader)? {
            let param = parse_param!(reader, |param| Some(param));
            Ok(Route {
                name_addr: addr,
                param,
            })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}
