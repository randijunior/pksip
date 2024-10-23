use crate::{
    macros::{alpha, space},
    message::SipMethod,
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;
#[derive(Debug, PartialEq, Eq)]
pub struct Allow<'a>(Vec<SipMethod<'a>>);

impl<'a> Allow<'a> {
    pub fn get(&self, index: usize) -> Option<&SipMethod<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for Allow<'a> {
    const NAME: &'static [u8] = b"Allow";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut allow: Vec<SipMethod> = Vec::new();
        let b_method = alpha!(scanner);
        let method = SipMethod::from(b_method);

        allow.push(method);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);

            let b_method = alpha!(scanner);
            let method = SipMethod::from(b_method);

            allow.push(method);
        }

        Ok(Allow(allow))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"INVITE, ACK, OPTIONS, CANCEL, BYE\r\n";
        let mut scanner = Scanner::new(src);
        let allow = Allow::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");

        assert_eq!(allow.get(0), Some(&SipMethod::Invite));
        assert_eq!(allow.get(1), Some(&SipMethod::Ack));
        assert_eq!(allow.get(2), Some(&SipMethod::Options));
        assert_eq!(allow.get(3), Some(&SipMethod::Cancel));
        assert_eq!(allow.get(4), Some(&SipMethod::Bye));
        assert_eq!(allow.get(5), None);
    }
}
