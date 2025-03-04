use std::fmt;

use reader::Reader;

use crate::{
    macros::parse_header_param,
    message::{NameAddr, Params},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Route` SIP header.
///
/// Specify the sequence of proxy servers and other intermediaries
/// that a SIP message should pass through on its way to the final destination.
#[derive(Debug, PartialEq, Eq)]
pub struct Route {
    pub(crate) addr: NameAddr,
    pub(crate) param: Option<Params>,
}

impl SipHeader<'_> for Route {
    const NAME: &'static str = "Route";
    /*
     * Route        =  "Route" HCOLON route-param *(COMMA route-param)
     * route-param  =  name-addr *( SEMI rr-param )
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let addr = parser::parse_name_addr(reader)?;
        let param = parse_header_param!(reader);
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
    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:bigbox3.site3.atlanta.com;lr>\r\n";
        let mut reader = Reader::new(src);
        let r = Route::parse(&mut reader);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host_port,
            HostPort {
                host: Host::DomainName("bigbox3.site3.atlanta.com".into()),
                port: None
            }
        );
        assert!(r.addr.uri.lr_param);

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut reader = Reader::new(src);
        let r = Route::parse(&mut reader);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host_port,
            HostPort {
                host: Host::DomainName("server10.biloxi.com".into()),
                port: None
            }
        );
        assert_eq!(r.param.unwrap().get("foo".into()), Some("bar"));
    }
}
