use crate::{
    macros::{parse_header_param, sip_parse_error},
    parser::Result,
    scanner::Scanner,
    uri::{NameAddr, Params, SipUri},
};

use crate::headers::SipHeader;

/// The `Route` SIP header.
///
/// Specify the sequence of proxy servers and other intermediaries
/// that a SIP message should pass through on its way to the final destination.
pub struct Route<'a> {
    pub(crate) addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for Route<'a> {
    const NAME: &'static str = "Route";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipUri::parse(scanner)? {
            let param = parse_header_param!(scanner);
            Ok(Route { addr, param })
        } else {
            sip_parse_error!("Invalid Route Header!")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::uri::{HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:bigbox3.site3.atlanta.com;lr>\r\n";
        let mut scanner = Scanner::new(src);
        let r = Route::parse(&mut scanner);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host,
            HostPort::DomainName {
                host: "bigbox3.site3.atlanta.com",
                port: None
            }
        );
        assert!(r.addr.uri.params.is_some());

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut scanner = Scanner::new(src);
        let r = Route::parse(&mut scanner);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host,
            HostPort::DomainName {
                host: "server10.biloxi.com",
                port: None
            }
        );
        assert_eq!(r.param.unwrap().get("foo"), Some(&"bar"));
    }
}
