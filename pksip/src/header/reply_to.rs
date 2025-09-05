use std::fmt;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::macros::parse_header_param;
use crate::message::{Parameters, SipAddr};
use crate::parser::Parser;

/// The `Reply-To` SIP header.
///
/// Contains a logical return URI that may be different from
/// the From header field
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReplyTo {
    uri: SipAddr,
    param: Option<Parameters>,
}

impl<'a> HeaderParser<'a> for ReplyTo {
    const NAME: &'static str = "Reply-To";

    /*
     * Reply-To      =  "Reply-To" HCOLON rplyto-spec
     * rplyto-spec   =  ( name-addr / addr-spec )
     *                  *( SEMI rplyto-param )
     * rplyto-param  =  generic-param
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let uri = parser.parse_sip_addr(false)?;
        let param = parse_header_param!(parser);

        Ok(ReplyTo { uri, param })
    }
}

impl fmt::Display for ReplyTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ReplyTo::NAME, self.uri)?;
        if let Some(param) = &self.param {
            write!(f, ";{}", param)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{DomainName, Host, HostPort, Scheme};

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>\r\n";
        let mut scanner = Parser::new(src);
        let reply_to = ReplyTo::parse(&mut scanner);
        let reply_to = reply_to.unwrap();

        assert_matches!(reply_to, ReplyTo {
            uri: SipAddr::NameAddr(addr),
            ..
        } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user.unwrap().user.as_ref(), "bob");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName(DomainName::new("biloxi.com")),
                    port: None
                }
            );
        });
    }
}
