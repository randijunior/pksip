use crate::{
    scanner::Scanner,
    macros::{parse_param, sip_parse_error},
    parser::{Result, SipParser},
    uri::{NameAddr, Params, SipUri},
};

use super::SipHeaderParser;

pub struct Route<'a> {
    pub(crate) name_addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for Route<'a> {
    const NAME: &'static [u8] = b"Route";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(scanner)? {
            let param = parse_param!(scanner, |param| Some(param));
            Ok(Route {
                name_addr: addr,
                param,
            })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}
