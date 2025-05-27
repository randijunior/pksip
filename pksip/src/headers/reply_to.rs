use std::fmt;

use crate::{
    error::Result,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::ParseCtx,
};

use crate::headers::SipHeaderParse;

/// The `Reply-To` SIP header.
///
/// Contains a logical return URI that may be different from
/// the From header field
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReplyTo<'a> {
    uri: SipUri<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParse<'a> for ReplyTo<'a> {
    const NAME: &'static str = "Reply-To";
    /*
     * Reply-To      =  "Reply-To" HCOLON rplyto-spec
     * rplyto-spec   =  ( name-addr / addr-spec )
     *                  *( SEMI rplyto-param )
     * rplyto-param  =  generic-param
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let uri = parser.parse_sip_uri(false)?;
        let param = parse_header_param!(parser);

        Ok(ReplyTo { uri, param })
    }
}

impl fmt::Display for ReplyTo<'_> {
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
    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>\r\n";
        let mut scanner = ParseCtx::new(src);
        let reply_to = ReplyTo::parse(&mut scanner);
        let reply_to = reply_to.unwrap();

        assert_matches!(reply_to, ReplyTo {
            uri: SipUri::NameAddr(addr),
            ..
        } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user.unwrap().user, "bob");
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
