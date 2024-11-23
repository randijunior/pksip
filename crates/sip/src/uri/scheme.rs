use reader::{until_byte, Reader};

use crate::{macros::sip_parse_error, parser::Result};

use super::{SIP, SIPS};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum Scheme {
    #[default]
    Sip,
    Sips,
}

impl Scheme {
    pub(crate) fn parse(reader: &mut Reader) -> Result<Self> {
        match until_byte!(reader, &b':') {
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
