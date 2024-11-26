use reader::Reader;

use crate::{
    headers::{self, EXPIRES_PARAM, Q_PARAM},
    macros::parse_header_param,
    msg::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

use super::Q;

#[derive(Debug, PartialEq, Eq)]
pub struct ContactUri<'a> {
    uri: SipUri<'a>,
    q: Option<Q>,
    expires: Option<u32>,
    param: Option<Params<'a>>,
}

/// The `Contact` SIP header.
///
/// Specifies the `URI` for the user or `UA` sending the message.
#[derive(Debug, PartialEq, Eq)]
pub enum Contact<'a> {
    Star,
    Uri(ContactUri<'a>),
}

impl<'a> SipHeader<'a> for Contact<'a> {
    const NAME: &'static str = "Contact";
    const SHORT_NAME: &'static str = "m";

    fn parse(reader: &mut Reader<'a>) -> Result<Contact<'a>> {
        if reader.peek() == Some(&b'*') {
            reader.next();
            return Ok(Contact::Star);
        }
        let uri = parser::parse_sip_uri(reader)?;
        let mut q = None;
        let mut expires = None;
        let param =
            parse_header_param!(reader, Q_PARAM = q, EXPIRES_PARAM = expires);
        let q = q.and_then(|q| Q::parse(q));
        let expires = expires.and_then(|expires| expires.parse().ok());

        Ok(Contact::Uri(ContactUri {
            uri,
            q,
            expires,
            param,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::msg::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"\"Mr. Watson\" <sip:watson@worcester.bell-telephone.com> \
        ;q=0.7; expires=3600\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri(ContactUri {
            uri: SipUri::NameAddr(addr),
            q,
            expires,
            ..
        }) => {
            assert_eq!(addr.display, Some("Mr. Watson"));
            assert_eq!(addr.uri.user.unwrap().user, "watson");
            assert_eq!(
                addr.uri.host,
                HostPort {
                    host: Host::DomainName("worcester.bell-telephone.com"),
                    port: None
                },
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(q, Some(Q(0, 7)));
            assert_eq!(expires, Some(3600));
        });

        let src =
            b"\"Mr. Watson\" <mailto:watson@bell-telephone.com> ;q=0.1\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);

        assert_matches!(contact, Err(err) => {
            assert_eq!(
                err.message,
                "Unsupported URI scheme: mailto".to_string()
            )
        });

        assert_eq!(reader.as_ref(), b":watson@bell-telephone.com> ;q=0.1\r\n");

        let src = b"sip:caller@u1.example.com\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri(ContactUri {
            uri: SipUri::Uri(uri),
            ..
        }) => {
            assert_eq!(uri.user.unwrap().user, "caller");
            assert_eq!(
                uri.host,
                HostPort {
                    host: Host::DomainName("u1.example.com"),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
        });
    }

    #[test]
    fn test_parse_ipv6_host() {
        let src = b"sips:[2620:0:2ef0:7070:250:60ff:fe03:32b7]";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri(ContactUri{
            uri: SipUri::Uri(uri),
            ..
        }) => {
            let addr: IpAddr =
            "2620:0:2ef0:7070:250:60ff:fe03:32b7".parse().unwrap();
        assert_eq!(
            uri.host,
            HostPort {
                host: Host::IpAddr(addr),
                port: None
            }
        );
        assert_eq!(uri.scheme, Scheme::Sips);
        });
    }

    #[test]
    fn test_parse_pass() {
        let src = b"sip:thks.ashwin:pass@212.123.1.213\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri(ContactUri {
            uri: SipUri::Uri(uri),
            ..
        }) => {
            assert_eq!(
                uri.host,
                HostPort {
                    host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(212, 123, 1, 213))),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
            let user = uri.user.unwrap();
            assert_eq!(user.user, "thks.ashwin");
            assert_eq!(user.pass, Some("pass"));
        });
    }

    #[test]
    fn test_parse_host_port() {
        let src = b"sip:192.168.1.1:5060";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri(ContactUri {
            uri: SipUri::Uri(uri),
            ..
        }) => {
            let addr = Ipv4Addr::new(192, 168, 1, 1);
            assert_eq!(
                uri.host,
                HostPort {
                    host: Host::IpAddr(IpAddr::V4(addr)),
                    port: Some(5060)
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
        });
    }
}
