use crate::{scanner::Scanner, macros::alpha, msg::SipMethod, parser::Result};

use super::SipHeaderParser;

pub struct Allow<'a>(Vec<SipMethod<'a>>);

impl<'a> SipHeaderParser<'a> for Allow<'a> {
    const NAME: &'static [u8] = b"Allow";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut allow: Vec<SipMethod> = Vec::new();
        let b_method = alpha!(scanner);
        let method = SipMethod::from(b_method);

        allow.push(method);

        while let Some(b',') = scanner.peek() {
            scanner.next();

            let b_method = alpha!(scanner);
            let method = SipMethod::from(b_method);

            allow.push(method);
        }

        Ok(Allow(allow))
    }
}
