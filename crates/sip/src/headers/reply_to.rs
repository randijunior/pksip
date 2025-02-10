use std::fmt;

use reader::Reader;

use crate::{
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Reply-To` SIP header.
///
/// Contains a logical return URI that may be different from the From header field
#[derive(Debug, PartialEq, Eq)]
pub struct ReplyTo {
    uri: SipUri,
    param: Option<Params>,
}

impl SipHeader<'_> for ReplyTo {
    const NAME: &'static str = "Reply-To";
    /*
     * Reply-To      =  "Reply-To" HCOLON rplyto-spec
     * rplyto-spec   =  ( name-addr / addr-spec )
     *                  *( SEMI rplyto-param )
     * rplyto-param  =  generic-param
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let uri = parser::parse_sip_uri(reader, false)?;
        let param = parse_header_param!(reader);

        Ok(ReplyTo { uri, param })
    }
}

impl fmt::Display for ReplyTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uri)?;
        if let Some(param) = &self.param {
            write!(f, ";{}", param)?;
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
        let src = b"Bob <sip:bob@biloxi.com>\r\n";
        let mut reader = Reader::new(src);
        let reply_to = ReplyTo::parse(&mut reader);
        let reply_to = reply_to.unwrap();

        assert_matches!(reply_to, ReplyTo {
            uri: SipUri::NameAddr(addr),
            ..
        } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user.unwrap().get_user(), "bob");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("biloxi.com".into()),
                    port: None
                }
            );
        });
    }
}
