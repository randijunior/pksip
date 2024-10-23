use crate::{
    macros::{read_until_byte, sip_parse_error},
    parser::SipParserError,
    scanner::Scanner,
};

use super::{SCHEME_SIP, SCHEME_SIPS};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

impl Scheme {
    pub(crate) fn parse(scanner: &mut Scanner) -> Result<Self, SipParserError> {
        match read_until_byte!(scanner, &b':') {
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
