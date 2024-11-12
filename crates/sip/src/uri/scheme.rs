use crate::{
    bytes::Bytes,
    macros::{until_byte, sip_parse_error},
    parser::SipParserError,
};

use super::{SCHEME_SIP, SCHEME_SIPS};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

impl Scheme {
    pub(crate) fn parse(bytes: &mut Bytes) -> Result<Self, SipParserError> {
        match until_byte!(bytes, &b':') {
            SCHEME_SIP => Ok(Scheme::Sip),
            SCHEME_SIPS => Ok(Scheme::Sips),
            // Unsupported URI scheme
            unsupported => sip_parse_error!(format!(
                "Unsupported URI scheme: {}",
                String::from_utf8_lossy(unsupported)
            )),
        }
    }
}
