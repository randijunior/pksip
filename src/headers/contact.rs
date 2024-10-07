use crate::{
    scanner::Scanner,
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
    const NAME: &'static [u8] = b"Contact";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"m");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if scanner.peek() == Some(&b'*') {
            scanner.next();
            return Ok(Contact::Star);
        }
        let uri = SipParser::parse_sip_uri(scanner)?;
        let mut q: Option<f32> = None;
        let mut expires: Option<u32> = None;
        let param = parse_param!(scanner, |param: Param<'a>| {
            let (name, value) = param;
            match name {
                Q_PARAM => {
                    q = Contact::parse_q_value(value);
                    None
                }
                EXPIRES_PARAM => {
                    if let Some(expires_param) = value {
                        expires = expires_param.parse().ok();
                        return None;
                    }
                    return Some(param);
                }
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
