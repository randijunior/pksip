use reader::Reader;

use crate::{
    headers::TAG_PARAM,
    macros::parse_header_param,
    msg::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

use std::{fmt, str};

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct To<'a> {
    pub uri: SipUri<'a>,
    pub tag: Option<&'a str>,
    pub params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for To<'a> {
    const NAME: &'static str = "To";
    const SHORT_NAME: &'static str = "t";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let uri = parser::parse_sip_uri(reader, false)?;
        let mut tag = None;
        let params = parse_header_param!(reader, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

impl fmt::Display for To<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)?;
        if let Some(tag) = self.tag {
            write!(f, ";tag={}", tag)?;
        }
        if let Some(params) = &self.params {
            write!(f, "{}", params)?;
        }

        Ok(())
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
                assert_eq!(addr.uri.user.unwrap().get_user(), "bob");
                assert_eq!(
                    addr.uri.host_port,
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
