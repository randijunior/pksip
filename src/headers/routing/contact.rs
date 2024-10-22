use crate::{
    headers::{self, EXPIRES_PARAM, Q_PARAM}, macros::parse_header_param, parser::Result, scanner::Scanner, uri::{Params, SipUri}
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq)]
pub enum Contact<'a> {
    Star,
    Uri {
        uri: SipUri<'a>,
        q: Option<f32>,
        expires: Option<u32>,
        param: Option<Params<'a>>,
    },
}

impl<'a> SipHeaderParser<'a> for Contact<'a> {
    const NAME: &'static [u8] = b"Contact";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"m");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        if scanner.peek() == Some(&b'*') {
            scanner.next();
            return Ok(Contact::Star);
        }
        let uri = SipUri::parse(scanner)?;
        let mut q = None;
        let mut expires = None;
        let param = parse_header_param!(scanner, Q_PARAM = q, EXPIRES_PARAM = expires);
        let q = q.and_then(|q| headers::parse_q(Some(q)));
        let expires = expires.and_then(|expires| expires.parse().ok());

        Ok(Contact::Uri {
            uri,
            q,
            expires,
            param,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::uri::{HostPort, Scheme, UserInfo};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"\"Mr. Watson\" <sip:watson@worcester.bell-telephone.com> \
        ;q=0.7; expires=3600\r\n";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);

        assert_matches!(contact, Ok(Contact::Uri { uri: SipUri::NameAddr(addr), q, expires, param }) => {
            assert_eq!(addr.display, Some("Mr. Watson"));
            assert_eq!(addr.uri.user, Some(UserInfo { user: "watson", password: None }));
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "worcester.bell-telephone.com", port: None });
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(q, Some(0.7));
            assert_eq!(expires, Some(3600));
            assert_eq!(param, None);
        });

        let src = b"*\r\n";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);

        assert_matches!(contact, Ok(Contact::Star));

        let src =
            b"\"Mr. Watson\" <mailto:watson@bell-telephone.com> ;q=0.1\r\n";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);
        assert_matches!(contact, Err(err) => {
            assert_eq!(err.message, "Unsupported URI scheme: mailto".to_string())
        });
        assert_eq!(scanner.as_ref(), b":watson@bell-telephone.com> ;q=0.1\r\n");

        let src = b"sip:caller@u1.example.com\r\n";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);
        assert_matches!(contact, Ok(Contact::Uri { uri: SipUri::Uri(uri), .. }) => {
            assert_eq!(uri.user, Some(UserInfo { user: "caller", password: None }));
            assert_eq!(uri.host, HostPort::DomainName { host: "u1.example.com", port: None });
            assert_eq!(uri.scheme, Scheme::Sip);
        });
    }

    #[test]
    fn test_parse_host_port() {
        let src = b"sip:192.168.1.1:5060";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);
        assert_matches!(contact, Ok(Contact::Uri { uri: SipUri::Uri(uri), .. }) => {
            assert_eq!(uri.host, HostPort::IpAddr {
                host: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                port: Some(5060)
            });
            assert_eq!(uri.scheme, Scheme::Sip);
        });
        let src = b"sips:[2620:0:2ef0:7070:250:60ff:fe03:32b7]";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);
        assert_matches!(contact, Ok(Contact::Uri { uri: SipUri::Uri(uri), .. }) => {
            let addr: IpAddr = "2620:0:2ef0:7070:250:60ff:fe03:32b7".parse().unwrap();
            assert_eq!(uri.host, HostPort::IpAddr {
                host: addr,
                port: None
            });
            assert_eq!(uri.scheme, Scheme::Sips);
        });

        let src = b"sip:thks.ashwin:pass@212.123.1.213\r\n";
        let mut scanner = Scanner::new(src);
        let contact = Contact::parse(&mut scanner);

        assert_matches!(contact, Ok(Contact::Uri { uri: SipUri::Uri(uri), .. }) => {
            assert_eq!(uri.host, HostPort::IpAddr {
                host: IpAddr::V4(Ipv4Addr::new(212, 123, 1, 213)),
                port: None
            });
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(uri.user, Some(UserInfo { user: "thks.ashwin", password: Some("pass") }));
        });
    }
}
