use crate::{
    macros::{parse_param, sip_parse_error},
    parser::SipParser,
    uri::{NameAddr, Params, SipUri},
};

use super::SipHeaderParser;

pub struct RecordRoute<'a> {
    addr: NameAddr<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for RecordRoute<'a> {
    const NAME: &'static [u8] = b"Record-Route";

    fn parse(reader: &mut crate::byte_reader::ByteReader<'a>) -> crate::parser::Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(reader)? {
            let param = parse_param!(reader, |param| Some(param));
            Ok(RecordRoute { addr, param })
        } else {
            sip_parse_error!("Invalid Record-Route Header!")
        }
    }
}
