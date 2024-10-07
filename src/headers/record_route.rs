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

    fn parse(scanner: &mut crate::scanner::Scanner<'a>) -> crate::parser::Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(scanner)? {
            let param = parse_param!(scanner, |param| Some(param));
            Ok(RecordRoute { addr, param })
        } else {
            sip_parse_error!("Invalid Record-Route Header!")
        }
    }
}
