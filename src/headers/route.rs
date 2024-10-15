use crate::{
    scanner::Scanner,
    macros::{parse_param, sip_parse_error},
    parser::{Result, SipParser},
    uri::{NameAddr, Params, SipUri},
};

use super::SipHeaderParser;

#[derive(Debug)]
pub struct Route<'a> {
    pub(crate) addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for Route<'a> {
    const NAME: &'static [u8] = b"Route";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(scanner)? {
            let param = parse_param!(scanner, |param| Some(param));
            Ok(Route {
                addr,
                param,
            })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::uri::{HostPort, Scheme, UriParams};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:bigbox3.site3.atlanta.com;lr>\r\n";
        let mut scanner = Scanner::new(src);
        let rr = Route::parse(&mut scanner);
        let rr = rr.unwrap();
        assert_matches!(rr, Route { addr, .. } => {
            assert_eq!(addr.display, None);
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user, None);
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "bigbox3.site3.atlanta.com", port: None });
            assert_eq!(addr.uri.params,Some(UriParams::default()));
        });

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut scanner = Scanner::new(src);
        let rr = Route::parse(&mut scanner);
        let rr = rr.unwrap();
        assert_matches!(rr, Route { addr, param } => {
            assert_eq!(addr.display, None);
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user, None);
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "server10.biloxi.com", port: None });
            assert_eq!(param,Some(Params::from(HashMap::from([(
                "foo",
                Some("bar")
            )]))));
        });
    }
}