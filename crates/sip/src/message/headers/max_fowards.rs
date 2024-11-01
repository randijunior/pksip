use crate::{
    bytes::Bytes,
    macros::{digits, sip_parse_error},
    parser::Result,
};

use crate::headers::SipHeader;

use std::str;

/// Limit the number of proxies or gateways that can forward the request.
pub struct MaxForwards(u32);

impl MaxForwards {
    pub fn new(m_fowards: u32) -> Self {
        Self(m_fowards)
    }
    pub fn max_fowards(&self) -> u32 {
        self.0
    }
}

impl<'a> SipHeader<'a> for MaxForwards {
    const NAME: &'static str = "Max-Forwards";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let digits = digits!(bytes);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(digits) => Ok(MaxForwards(digits)),
            Err(_) => sip_parse_error!("invalid Max Fowards"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let src = b"6\r\n";
        let mut bytes = Bytes::new(src);
        let c_length = MaxForwards::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(c_length.0, 6)
    }
}
