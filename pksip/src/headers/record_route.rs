use std::fmt;

use crate::{
    error::Result,
    macros::parse_header_param,
    message::{NameAddr, Params},
    parser::Parser,
};

use crate::headers::SipHeaderParse;

/// The `Record-Route` SIP header.
///
/// Keeps proxies in the signaling path for consistent
/// routing and session control.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RecordRoute<'a> {
    /// The address of the record route.
    pub addr: NameAddr<'a>,
    /// Optional parameters associated with the record route.
    pub params: Option<Params<'a>>,
}

impl<'a> SipHeaderParse<'a> for RecordRoute<'a> {
    const NAME: &'static str = "Record-Route";
    /*
     * Record-Route  =  "Record-Route" HCOLON rec-route *(COMMA rec-route)
     * rec-route     =  name-addr *( SEMI rr-param )
     * rr-param      =  generic-param
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let addr = parser.parse_name_addr()?;
        let params = parse_header_param!(parser);
        Ok(RecordRoute { addr, params })
    }
}

impl fmt::Display for RecordRoute<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", RecordRoute::NAME, self.addr)?;
        if let Some(param) = &self.params {
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
        let src = b"<sip:server10.biloxi.com;lr>\r\n";
        let mut scanner = Parser::new(src);
        let rr = RecordRoute::parse(&mut scanner);
        let rr = rr.unwrap();

        assert_eq!(rr.addr.display, None);
        assert_eq!(rr.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            rr.addr.uri.host_port,
            HostPort {
                host: Host::DomainName("server10.biloxi.com".into()),
                port: None
            }
        );
        assert!(rr.addr.uri.lr_param);

        let src = b"<sip:bigbox3.site3.atlanta.com;lr>;foo=bar\r\n";
        let mut scanner = Parser::new(src);
        let rr = RecordRoute::parse(&mut scanner);
        let rr = rr.unwrap();

        assert_eq!(rr.addr.display, None);
        assert_eq!(rr.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            rr.addr.uri.host_port,
            HostPort {
                host: Host::DomainName("bigbox3.site3.atlanta.com".into()),
                port: None
            }
        );
        assert_eq!(rr.params.unwrap().get("foo").unwrap(), Some("bar"));
    }
}
