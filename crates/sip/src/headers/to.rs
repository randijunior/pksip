use reader::Reader;

use crate::{
    headers::TAG_PARAM,
    macros::parse_header_param,
    msg::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

use std::str;

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
#[derive(Debug, PartialEq, Eq)]
pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for To<'a> {
    const NAME: &'static str = "To";
    const SHORT_NAME: &'static str = "t";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let uri = parser::parse_sip_uri(reader)?;
        let mut tag = None;
        let params = parse_header_param!(reader, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>;tag=a6c85cf\r\n";
        let mut reader = Reader::new(src);
        let to = To::parse(&mut reader);
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
                    HostPort {
                        host: Host::DomainName("biloxi.com"),
                        port: None
                    }
                );
                assert_eq!(tag, Some("a6c85cf"));
            }
            _ => unreachable!(),
        }
    }
}
