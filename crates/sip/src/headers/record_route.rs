use std::fmt;

use reader::Reader;

use crate::{
    macros::parse_header_param,
    message::{NameAddr, Params},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Record-Route` SIP header.
///
/// Keeps proxies in the signaling path for consistent routing and session control.
#[derive(Debug, PartialEq, Eq)]
pub struct RecordRoute {
    addr: NameAddr,
    param: Option<Params>,
}

impl SipHeader<'_> for RecordRoute {
    const NAME: &'static str = "Record-Route";
    /*
     * Record-Route  =  "Record-Route" HCOLON rec-route *(COMMA rec-route)
     * rec-route     =  name-addr *( SEMI rr-param )
     * rr-param      =  generic-param
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        let addr = parser::parse_name_addr(reader)?;
        let param = parse_header_param!(reader);
        Ok(RecordRoute { addr, param })
    }
}

impl fmt::Display for RecordRoute {
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
        let src = b"<sip:server10.biloxi.com;lr>\r\n";
        let mut reader = Reader::new(src);
        let rr = RecordRoute::parse(&mut reader);
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
        assert!(rr.addr.uri.lr_param.is_some());

        let src = b"<sip:bigbox3.site3.atlanta.com;lr>;foo=bar\r\n";
        let mut reader = Reader::new(src);
        let rr = RecordRoute::parse(&mut reader);
        let rr = rr.unwrap();

        assert_eq!(rr.addr.display, None);
        assert_eq!(rr.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            rr.addr.uri.host_port,
            HostPort {
                host: Host::DomainName(
                    "bigbox3.site3.atlanta.com".into()
                ),
                port: None
            }
        );
        assert_eq!(rr.param.unwrap().get("foo".into()), Some("bar"));
    }
}
