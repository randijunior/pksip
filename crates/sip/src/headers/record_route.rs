use reader::Reader;

use crate::{
    macros::parse_header_param,
    msg::{NameAddr, Params},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Record-Route` SIP header.
///
/// Keeps proxies in the signaling path for consistent routing and session control.
#[derive(Debug, PartialEq, Eq)]
pub struct RecordRoute<'a> {
    addr: NameAddr<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for RecordRoute<'a> {
    const NAME: &'static str = "Record-Route";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let addr = parser::parse_name_addr(reader)?;
        let param = parse_header_param!(reader);
        Ok(RecordRoute { addr, param })
    }
}

#[cfg(test)]
mod tests {

    use crate::msg::{Host, HostPort, Scheme};

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
            rr.addr.uri.host,
            HostPort {
                host: Host::DomainName("server10.biloxi.com"),
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
            rr.addr.uri.host,
            HostPort {
                host: Host::DomainName("bigbox3.site3.atlanta.com"),
                port: None
            }
        );
        assert_eq!(rr.param.unwrap().get("foo"), Some(&"bar"));
    }
}
