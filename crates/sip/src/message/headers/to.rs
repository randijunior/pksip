use crate::{
    bytes::Bytes,
    headers::TAG_PARAM,
    macros::parse_header_param,
    parser::Result,
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;

use std::str;

pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for To<'a> {
    const NAME: &'static [u8] = b"To";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"t");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let uri = SipUri::parse(bytes)?;
        let mut tag = None;
        let params = parse_header_param!(bytes, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

#[cfg(test)]
mod tests {
    use crate::uri::{HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>;tag=a6c85cf\r\n";
        let mut bytes = Bytes::new(src);
        let to = To::parse(&mut bytes);
        let to = to.unwrap();

        match to {
            To {
                uri: SipUri::NameAddr(addr),
                tag,
                ..
            } => {
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(addr.display, Some("Bob"));
                assert_eq!(addr.uri.user.unwrap().user, "bob");
                assert_eq!(
                    addr.uri.host,
                    HostPort::DomainName {
                        host: "biloxi.com",
                        port: None
                    }
                );
                assert_eq!(tag, Some("a6c85cf"));
            }
            _ => unreachable!(),
        }
    }
}
