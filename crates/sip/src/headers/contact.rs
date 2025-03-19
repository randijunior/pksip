use super::{Header, ParseHeaderError};
use crate::{
    headers::{SipHeader, EXPIRES_PARAM, Q_PARAM},
    internal::Q,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::{self, Result},
};
use core::fmt;
use reader::Reader;

/// The `Contact` SIP header.
///
/// Specifies the `URI` for the user or `UA` sending the message.
///
/// # Examples
///
/// ```
/// # use sip::headers::contact::Contact;
/// # use sip::message::{HostPort, Host, UserInfo, UriBuilder, SipUri, NameAddr};
/// let uri = SipUri::NameAddr(NameAddr {
///     display: None,
///     uri: UriBuilder::new()
///         .user(UserInfo::new("alice", None))
///         .host(HostPort::from(Host::DomainName("client.atlanta.example.com".into())))
///         .get()
/// });
/// let c = Contact::Uri {
///     uri,
///     q: None,
///     expires: None,
///     param: None
/// };
///
/// assert_eq!("Contact: <sip:alice@client.atlanta.example.com>".as_bytes().try_into(), Ok(c));
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum Contact {
    Uri {
        uri: SipUri,
        q: Option<Q>,
        expires: Option<u32>,
        param: Option<Params>,
    },
    Star,
}

impl Contact {
    pub fn uri(&self) -> Option<&SipUri> {
        if let Contact::Uri { uri, .. } = self {
            Some(uri)
        } else {
            None
        }
    }
}

impl SipHeader<'_> for Contact {
    const NAME: &'static str = "Contact";
    const SHORT_NAME: &'static str = "m";
    /*
     * Contact        =  ("Contact" / "m" ) HCOLON
     *                   ( STAR / (contact-param *(COMMA contact-param)))
     * contact-param  =  (name-addr / addr-spec) *(SEMI contact-params)
     * name-addr      =  [ display-name ] LAQUOT addr-spec RAQUOT
     * addr-spec      =  SIP-URI / SIPS-URI / absoluteURI
     * display-name   =  *(token LWS)/ quoted-string
     *
     * contact-params     =  c-p-q / c-p-expires
     *                       / contact-extension
     * c-p-q              =  "q" EQUAL qvalue
     * c-p-expires        =  "expires" EQUAL delta-seconds
     * contact-extension  =  generic-param
     * delta-seconds      =  1*DIGIT
     */
    fn parse(reader: &mut Reader) -> Result<Self> {
        if reader.peek() == Some(&b'*') {
            reader.next();
            return Ok(Contact::Star);
        }
        let uri = parser::parse_sip_uri(reader, false)?;
        let mut q = None;
        let mut expires = None;
        let param = parse_header_param!(
            reader,
            Q_PARAM = q,
            EXPIRES_PARAM = expires
        );

        let q = q.map(|q| q.parse()).transpose()?;
        let expires =
            expires.and_then(|expires| expires.parse().ok());

        Ok(Contact::Uri {
            uri,
            q,
            expires,
            param,
        })
    }
}

impl TryFrom<&[u8]> for Contact {
    type Error = ParseHeaderError;

    fn try_from(
        value: &[u8],
    ) -> std::result::Result<Self, Self::Error> {
        Header::from_bytes(value)?
            .into_contact()
            .map_err(|_| ParseHeaderError(Self::NAME))
    }
}

impl fmt::Display for Contact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Contact::Star => write!(f, "*"),
            Contact::Uri {
                uri,
                q,
                expires,
                param,
            } => {
                write!(f, "{}", uri)?;

                if let Some(q) = q {
                    write!(f, "{}", q)?;
                }
                if let Some(expires) = expires {
                    write!(f, "{}", expires)?;
                }
                if let Some(param) = &param {
                    write!(f, ";{}", param)?;
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"\"Mr. Watson\" <sip:watson@worcester.bell-telephone.com> \
        ;q=0.7; expires=3600\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri {
            uri: SipUri::NameAddr(addr),
            q,
            expires,
            ..
        } => {
            assert_eq!(addr.display, Some("Mr. Watson".into()));
            assert_eq!(addr.uri.user.unwrap().get_user(), "watson");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("worcester.bell-telephone.com".into()),
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

        assert_eq!(
            reader.as_ref(),
            b":watson@bell-telephone.com> ;q=0.1\r\n"
        );

        let src = b"sip:caller@u1.example.com\r\n";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri {
            uri: SipUri::Uri(uri),
            ..
        } => {
            assert_eq!(uri.user.unwrap().get_user(), "caller");
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::DomainName("u1.example.com".into()),
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

        assert_matches!(contact, Contact::Uri {
            uri: SipUri::Uri(uri),
            ..
        } => {
            let addr: IpAddr =
            "2620:0:2ef0:7070:250:60ff:fe03:32b7".parse().unwrap();
        assert_eq!(
            uri.host_port,
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

        assert_matches!(contact, Contact::Uri {
            uri: SipUri::Uri(uri),
            ..
        } => {
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::IpAddr(IpAddr::V4(Ipv4Addr::new(212, 123, 1, 213))),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
            let user = uri.user.unwrap();
            assert_eq!(user.get_user(), "thks.ashwin");
            assert_eq!(user.get_pass(), Some("pass"));
        });
    }

    #[test]
    fn test_parse_host_port() {
        let src = b"sip:192.168.1.1:5060";
        let mut reader = Reader::new(src);
        let contact = Contact::parse(&mut reader);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact::Uri {
            uri: SipUri::Uri(uri),
            ..
        } => {
            let addr = Ipv4Addr::new(192, 168, 1, 1);
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::IpAddr(IpAddr::V4(addr)),
                    port: Some(5060)
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
        });
    }
}
