use crate::{byte_reader::ByteReader, parser::{SipParser, EXPIRES_PARAM, Q_PARAM, Result}, uri::{GenericParams, SipUri}};

use super::SipHeaderParser;

use std::str;

pub struct Contact<'a> {
    uri: SipUri<'a>,
    q: Option<f32>,
    expires: Option<u32>,
    other_params: Option<GenericParams<'a>>,
}

impl<'a> SipHeaderParser<'a> for Contact<'a> {
    const NAME: &'a [u8] = b"Contact";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Contact<'a>> {
        let uri = SipParser::parse_sip_uri(reader)?;
        let mut q: Option<f32> = None;
        let mut expires: Option<u32> = None;
        let mut params = GenericParams::new();
        while let Some(&b';') = reader.peek() {
            reader.next();
            let (name, value) = Contact::parse_param(reader)?;
            match name {
                Q_PARAM => if let Some(q_param) = value {
                    q = q_param.parse().ok();
                },
                EXPIRES_PARAM => if let Some(expires_param) = value {
                    expires = expires_param.parse().ok();
                },
                _=> {
                    params.set(str::from_utf8(name)?, value);
                }
            }
        }
        let other_params = if params.is_empty() {
            None
        } else {
            Some(params)
        };

        Ok(Contact { uri, q, expires, other_params })
    }
}
