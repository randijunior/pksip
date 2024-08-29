use crate::{
    byte_reader::ByteReader,
    parser::{Result, SipParser},
    uri::{Params, SipUri},
};

use super::SipHeaderParser;

use std::str;

pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) other_params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for To<'a> {
    const NAME: &'a [u8] = b"From";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"f");

    fn parse(reader: &mut ByteReader<'a>) -> Result<To<'a>> {
        let uri = SipParser::parse_sip_uri(reader)?;
        let (tag, other_params) = SipParser::parse_fromto_param(reader)?;

        Ok(To {
            tag,
            uri,
            other_params,
        })
    }
}
