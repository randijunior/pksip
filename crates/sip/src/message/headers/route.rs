use crate::{
    bytes::Bytes,
    macros::{parse_param, sip_parse_error},
    parser::Result,
    uri::{NameAddr, Params, SipUri},
};

use crate::headers::SipHeader;

/// Specify the sequence of proxy servers and other intermediaries
/// that a SIP message should pass through on its way to the final destination.
pub struct Route<'a> {
    pub(crate) addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for Route<'a> {
    const NAME: &'static str = "Route";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipUri::parse(bytes)? {
            let param = parse_param!(bytes);
            Ok(Route { addr, param })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::uri::{HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:bigbox3.site3.atlanta.com;lr>\r\n";
        let mut bytes = Bytes::new(src);
        let rr = Route::parse(&mut bytes);
        let rr = rr.unwrap();
        match rr {
            Route { addr, .. } => {
                assert_eq!(addr.display, None);
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(
                    addr.uri.host,
                    HostPort::DomainName {
                        host: "bigbox3.site3.atlanta.com",
                        port: None
                    }
                );
                assert!(addr.uri.params.is_some());
            }
            _ => unreachable!(),
        }

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut bytes = Bytes::new(src);
        let rr = Route::parse(&mut bytes);
        let rr = rr.unwrap();
        match rr {
            Route { addr, param } => {
                assert_eq!(addr.display, None);
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(
                    addr.uri.host,
                    HostPort::DomainName {
                        host: "server10.biloxi.com",
                        port: None
                    }
                );
                assert_eq!(param.unwrap().get("foo"), Some(&"bar"));
            }
            _ => unreachable!(),
        }
    }
}
