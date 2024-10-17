use crate::{
    macros::{parse_param, sip_parse_error},
    parser::{Result, SipParser},
    scanner::Scanner,
    uri::{NameAddr, Params, SipUri},
};

use crate::headers::SipHeaderParser;


#[derive(Debug, PartialEq, Eq)]
pub struct RecordRoute<'a> {
    addr: NameAddr<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for RecordRoute<'a> {
    const NAME: &'static [u8] = b"Record-Route";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipParser::parse_sip_uri(scanner)? {
            let param = parse_param!(scanner, |param| Some(param));
            Ok(RecordRoute { addr, param })
        } else {
            sip_parse_error!("Invalid Record-Route Header!")
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
        let src = b"<sip:server10.biloxi.com;lr>\r\n";
        let mut scanner = Scanner::new(src);
        let rr = RecordRoute::parse(&mut scanner);
        let rr = rr.unwrap();
        assert_matches!(rr, RecordRoute { addr, .. } => {
            assert_eq!(addr.display, None);
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user, None);
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "server10.biloxi.com", port: None });
            assert_eq!(addr.uri.params,Some(UriParams::default()));
        });

        let src = b"<sip:bigbox3.site3.atlanta.com;lr>;foo=bar\r\n";
        let mut scanner = Scanner::new(src);
        let rr = RecordRoute::parse(&mut scanner);
        let rr = rr.unwrap();
        assert_matches!(rr, RecordRoute { addr, param } => {
            assert_eq!(addr.display, None);
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user, None);
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "bigbox3.site3.atlanta.com", port: None });
            assert_eq!(param,Some(Params::from(HashMap::from([(
                "foo",
                Some("bar")
            )]))));
        });
    }
}
