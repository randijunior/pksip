use crate::{
    byte_reader::ByteReader,
    macros::parse_param,
    parser::{Param, Result, SipParser, EXPIRES_PARAM, Q_PARAM},
    uri::{Params, SipUri},
};

use super::SipHeaderParser;

pub enum Contact<'a> {
    Star,
    Uri {
        uri: SipUri<'a>,
        q: Option<f32>,
        expires: Option<u32>,
        param: Option<Params<'a>>,
    },
}

impl<'a> SipHeaderParser<'a> for Contact<'a> {
    const NAME: &'a [u8] = b"Contact";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        if reader.peek() == Some(&b'*') {
            reader.next();
            return Ok(Contact::Star);
        }
        let uri = SipParser::parse_sip_uri(reader)?;
        let mut q: Option<f32> = None;
        let mut expires: Option<u32> = None;
        let param = parse_param!(reader, |param: Param<'a>| {
            let (name, value) = param;
            match name {
                Q_PARAM => {
                    q = Contact::parse_q_value(value);
                    None
                },
                EXPIRES_PARAM => {
                    if let Some(expires_param) = value {
                        expires = expires_param.parse().ok();
                        return None
                    }
                    return Some(param)
                },
                _ => Some(param),
            }
        });

        Ok(Contact::Uri {
            uri,
            q,
            expires,
            param,
        })
    }
}
