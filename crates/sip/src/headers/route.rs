use reader::Reader;

use crate::{
    macros::{parse_header_param, sip_parse_error},
    msg::{NameAddr, Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Route` SIP header.
///
/// Specify the sequence of proxy servers and other intermediaries
/// that a SIP message should pass through on its way to the final destination.
#[derive(Debug, PartialEq, Eq)]
pub struct Route<'a> {
    pub(crate) addr: NameAddr<'a>,
    pub(crate) param: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for Route<'a> {
    const NAME: &'static str = "Route";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let addr = parser::parse_name_addr(reader)?;
        let param = parse_header_param!(reader);
        Ok(Route { addr, param })
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{Host, HostPort, Scheme};

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
            r.addr.uri.host,
            HostPort {
                host: Host::DomainName("bigbox3.site3.atlanta.com"),
                port: None
            }
        );
        assert!(r.addr.uri.lr_param.is_some());

        let src = b"<sip:server10.biloxi.com;lr>;foo=bar\r\n";
        let mut reader = Reader::new(src);
        let r = Route::parse(&mut reader);
        let r = r.unwrap();

        assert_eq!(r.addr.display, None);
        assert_eq!(r.addr.uri.scheme, Scheme::Sip);
        assert_eq!(
            r.addr.uri.host,
            HostPort {
                host: Host::DomainName("server10.biloxi.com"),
                port: None
            }
        );
        assert_eq!(r.param.unwrap().get("foo"), Some(&"bar"));
    }
}
