use crate::{
    byte_reader::ByteReader,
    macros::sip_parse_error,
    parser::{Result, SipParser},
    uri::{Params, NameAddr, SipUri},
};

use super::SipHeaderParser;

use std::str;

pub struct Route<'a> {
    pub(crate) name_addr: NameAddr<'a>,
    pub(crate) other_param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for Route<'a> {
    const NAME: &'a [u8] = b"Route";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Route<'a>> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(reader)? {
            let mut params = Params::new();
            while let Some(&b';') = reader.peek() {
                let (name, value) = Route::parse_param(reader)?;
                params.set(str::from_utf8(name)?, value);
            }

            let params = if params.is_empty() {
                None
            } else {
                Some(params)
            };
            Ok(Route {
                name_addr: addr,
                other_param: params,
            })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}
