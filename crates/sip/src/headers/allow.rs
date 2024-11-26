use reader::{alpha, Reader};

use crate::{macros::hdr_list, msg::SipMethod, parser::Result};

use crate::headers::SipHeader;
/// The `Allow` SIP header
///
/// Indicates what methods is supported by the `UA`.
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

impl<'a> SipHeader<'a> for Allow<'a> {
    const NAME: &'static str = "Allow";

    fn parse(reader: &mut Reader<'a>) -> Result<Allow<'a>> {
        let allow = hdr_list!(reader => {
            let b_method = alpha!(reader);

            SipMethod::from(b_method)
        });

        Ok(Allow(allow))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"INVITE, ACK, OPTIONS, CANCEL, BYE\r\n";
        let mut reader = Reader::new(src);
        let allow = Allow::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_eq!(allow.get(0), Some(&SipMethod::Invite));
        assert_eq!(allow.get(1), Some(&SipMethod::Ack));
        assert_eq!(allow.get(2), Some(&SipMethod::Options));
        assert_eq!(allow.get(3), Some(&SipMethod::Cancel));
        assert_eq!(allow.get(4), Some(&SipMethod::Bye));
        assert_eq!(allow.get(5), None);
    }
}
