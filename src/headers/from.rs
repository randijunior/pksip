use crate::{
    byte_reader::ByteReader,
    parser::{Result, SipParser},
    uri::{Params, SipUri},
};

use super::SipHeaderParser;

use std::str;

pub struct From<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) other_params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for From<'a> {
    const NAME: &'a [u8] = b"From";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"f");

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let uri = SipParser::parse_sip_uri(reader)?;
        let (tag, other_params) = SipParser::parse_fromto_param(reader)?;

        Ok(From {
            tag,
            uri,
            other_params,
        })
    }
}
