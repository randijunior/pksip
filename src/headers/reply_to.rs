use crate::{
    scanner::Scanner,
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
    const NAME: &'static [u8] = b"Reply-To";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let uri = SipParser::parse_sip_uri(scanner)?;
        let param = parse_param!(scanner, |param| Some(param));

        Ok(ReplyTo { uri, param })
    }
}
