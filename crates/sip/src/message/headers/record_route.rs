use crate::{
    bytes::Bytes,
    macros::{parse_header_param, sip_parse_error},
    parser::Result,
    uri::{NameAddr, Params, SipUri},
};

use crate::headers::SipHeaderParser;

pub struct RecordRoute<'a> {
    addr: NameAddr<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for RecordRoute<'a> {
    const NAME: &'static [u8] = b"Record-Route";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        if let SipUri::NameAddr(addr) = SipUri::parse(bytes)? {
            let param = parse_header_param!(bytes);
            Ok(RecordRoute { addr, param })
        } else {
            sip_parse_error!("Invalid Record-Route Header!")
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::uri::{HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<sip:server10.biloxi.com;lr>\r\n";
        let mut bytes = Bytes::new(src);
        let rr = RecordRoute::parse(&mut bytes);
        let rr = rr.unwrap();
        match rr {
            RecordRoute { addr, .. } => {
                assert_eq!(addr.display, None);
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(
                    addr.uri.host,
                    HostPort::DomainName {
                        host: "server10.biloxi.com",
                        port: None
                    }
                );
                assert!(addr.uri.params.is_some());
            }
            _ => unreachable!(),
        }

        let src = b"<sip:bigbox3.site3.atlanta.com;lr>;foo=bar\r\n";
        let mut bytes = Bytes::new(src);
        let rr = RecordRoute::parse(&mut bytes);
        let rr = rr.unwrap();

        match rr {
            RecordRoute { addr, param } => {
                assert_eq!(addr.display, None);
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(
                    addr.uri.host,
                    HostPort::DomainName {
                        host: "bigbox3.site3.atlanta.com",
                        port: None
                    }
                );
                assert_eq!(param.unwrap().get("foo"), Some(&"bar"));
            }
            _ => unreachable!(),
        }
    }
}
