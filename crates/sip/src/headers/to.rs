use reader::Reader;

use crate::{
    headers::TAG_PARAM,
    internal::ArcStr,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::{self, Result, SipParserError},
};

use crate::headers::SipHeader;

use std::{
    fmt,
    str::{self, FromStr},
};

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct To {
    pub uri: SipUri,
    pub tag: Option<ArcStr>,
    pub params: Option<Params>,
}

impl SipHeader<'_> for To {
    const NAME: &'static str = "To";
    const SHORT_NAME: &'static str = "t";
    /*
     * To        =  ( "To" / "t" ) HCOLON ( name-addr
     *              / addr-spec ) *( SEMI to-param )
     * to-param  =  tag-param / generic-param
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let uri = parser::parse_sip_uri(reader, false)?;
        let mut tag = None;
        let params = parse_header_param!(reader, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

impl FromStr for To {
    type Err = SipParserError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(&mut Reader::new(s.as_bytes()))
    }
}

impl fmt::Display for To {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)?;
        if let Some(tag) = &self.tag {
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
    use crate::message::{Host, HostPort, Scheme};

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
                assert_eq!(addr.display, Some("Bob".into()));
                assert_eq!(addr.uri.user.unwrap().get_user(), "bob");
                assert_eq!(
                    addr.uri.host_port,
                    HostPort {
                        host: Host::DomainName("biloxi.com".into()),
                        port: None
                    }
                );
                assert_eq!(tag, Some("a6c85cf".into()));
            }
            _ => unreachable!(),
        }
    }
}
