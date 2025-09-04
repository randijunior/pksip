use std::fmt;

use crate::error::Result;
use crate::header::HeaderParser;
use crate::macros::parse_header_param;
use crate::message::NameAddr;
use crate::message::Parameters;
use crate::parser::Parser;

/// The `Route` SIP header.
///
/// Specify the sequence of proxy servers and other
/// intermediaries that a SIP message should pass through on
/// its way to the final destination.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Route {
    pub(crate) addr: NameAddr,
    pub(crate) param: Option<Parameters>,
}

impl<'a> HeaderParser<'a> for Route {
    const NAME: &'static str = "Route";

    /*
     * Route        =  "Route" HCOLON route-param *(COMMA
     * route-param) route-param  =  name-addr *( SEMI
     * rr-param )
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let addr = parser.parse_name_addr()?;
        let param = parse_header_param!(parser);
        Ok(Route { addr, param })
    }
}

impl fmt::Display for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.addr)?;

        if let Some(param) = &self.param {
            write!(f, ";{}", param)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::DomainName;
    use crate::message::Host;
    use crate::message::HostPort;
    use crate::message::Scheme;

    #[test]
    fn test_parse() {
        let src = b"<sip:bigbox3.site3.atlanta.com;lr>\r\n";
        let mut scanner = Parser::new(src);
        let r = Route::parse(&mut scanner);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host_port,
            HostPort {
                host: Host::DomainName(DomainName::new("bigbox3.site3.atlanta.com")),
                port: None
            }
        );
        assert!(r.addr.uri.lr_param);

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut scanner = Parser::new(src);
        let r = Route::parse(&mut scanner);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host_port,
            HostPort {
                host: Host::DomainName(DomainName::new("server10.biloxi.com")),
                port: None
            }
        );
        assert_eq!(r.param.unwrap().get_named("foo"), Some("bar"));
    }
}
