use crate::{
    bytes::Bytes,
    macros::{sip_parse_error, until_byte},
    parser::Result,
};

use super::{SIP, SIPS};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

impl Scheme {
    pub(crate) fn parse(bytes: &mut Bytes) -> Result<Self> {
        match until_byte!(bytes, &b':') {
            SIP => Ok(Self::Sip),
            SIPS => Ok(Self::Sips),
            // Unsupported URI scheme
            other => sip_parse_error!(format!(
                "Unsupported URI scheme: {}",
                String::from_utf8_lossy(other)
            )),
        }
    }
}
