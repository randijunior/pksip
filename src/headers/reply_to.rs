use crate::{
    byte_reader::ByteReader,
    macros::parse_param,
    parser::{Result, SipParser},
    uri::{Params, SipUri},
};

use super::SipHeaderParser;

pub struct ReplyTo<'a> {
    uri: SipUri<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ReplyTo<'a> {
    const NAME: &'a [u8] = b"Reply-To";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let uri = SipParser::parse_sip_uri(reader)?;
        let param = parse_param!(reader, |param| Some(param));

        Ok(ReplyTo { uri, param })
    }
}
