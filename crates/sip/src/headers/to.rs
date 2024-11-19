use crate::{
    headers::TAG_PARAM,
    macros::parse_header_param,
    parser::Result,
    scanner::Scanner,
    uri::{Params, SipUri},
};

use crate::headers::SipHeader;

use std::str;

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for To<'a> {
    const NAME: &'static str = "To";
    const SHORT_NAME: Option<&'static str> = Some("t");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let uri = SipUri::parse(scanner)?;
        let mut tag = None;
        let params = parse_header_param!(scanner, TAG_PARAM = tag);

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
        let mut scanner = Scanner::new(src);
        let to = To::parse(&mut scanner);
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
