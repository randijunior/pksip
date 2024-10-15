use crate::{
    scanner::Scanner,
    parser::{Result, SipParser},
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;

use std::str;

pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for To<'a> {
    const NAME: &'static [u8] = b"From";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"f");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let uri = SipParser::parse_sip_uri(scanner)?;
        let (tag, params) = SipParser::parse_fromto_param(scanner)?;

        Ok(To { tag, uri, params })
    }
}
