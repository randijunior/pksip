use crate::{
    error::Result,
    headers::{SipHeaderParse, EXPIRES_PARAM, Q_PARAM},
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::ParseCtx,
    Q,
};
use core::fmt;

/// The `Contact` SIP header.
///
/// Specifies the `URI` for the user or `UA` sending the
/// message.
///
/// # Examples
///
/// ```
/// # use pksip::headers::Contact;
/// # use pksip::message::{HostPort, Host, UriUser, UriBuilder, SipUri, NameAddr};
/// let uri = SipUri::from_static("<sip:alice@client.atlanta.example.com>").unwrap();
/// 
/// let c = Contact {
///     uri,
///     q: None,
///     expires: None,
///     param: None,
/// };
///
/// assert_eq!(
///     "Contact: <sip:alice@client.atlanta.example.com>",
///     c.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Contact<'a> {
    /// The URI of the contact.
    pub uri: SipUri<'a>,
    /// The quality value of the contact.
    pub q: Option<Q>,
    /// The expires parameter of the contact.
    pub expires: Option<u32>,
    /// Additional parameters.
    pub param: Option<Params<'a>>,
}

impl<'a> Contact<'a> {
    /// Parse a `To` header instance from a `&str`.
    pub fn from_str(s: &'a str) -> Result<Self> {
        Self::parse(&mut ParseCtx::new(s.as_bytes()))
    }
}

impl<'a> SipHeaderParse<'a> for Contact<'a> {
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
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let uri = parser.parse_sip_uri(false)?;
        let mut q = None;
        let mut expires = None;
        let param = parse_header_param!(parser, Q_PARAM = q, EXPIRES_PARAM = expires);

        let q = q.map(|q| q.parse()).transpose()?;
        let expires = expires.and_then(|expires| expires.parse().ok());

        Ok(Contact { uri, q, expires, param })
    }
}

impl fmt::Display for Contact<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: ", Contact::NAME)?;

        write!(f, "{}", self.uri)?;

        if let Some(q) = self.q {
            write!(f, "{}", q)?;
        }
        if let Some(expires) = self.expires {
            write!(f, "{}", expires)?;
        }
        if let Some(param) = &self.param {
            write!(f, ";{}", param)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    // ContactHeader inputs

    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "Contact : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "Contact  : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "Contact\t: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "Contact:\n  \"Alice Liddell\" \n\t<sip:alice@wonderland.com>"
    // "m: Alice <sip:alice@wonderland.com>"
    // "Contact: *"
    // "Contact: \t  *"
    // "M: *"
    // "Contact: \"John\" *"
    // "Contact: \"John\" <*>"
    // "Contact: *;foo=bar"
    // "Contact: Alice sip:alice@wonderland.com>"
    // "Contact:"
    // "Contact: "
    // "Contact:\t"
    // "Contact: foo"
    // "Contact: foo bar"
    // "Contact: \"Alice\" sip:alice@wonderland.com>"
    // "Contact: \"<Alice>\" sip:alice@wonderland.com>"
    // "Contact: \"sip:alice@wonderland.com\""
    // "Contact: \"sip:alice@wonderland.com\"  <sip:alice@wonderland.com>"
    // "Contact: \"<sip:alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "Contact: \"<sip: alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "cOntACt: \"Alice Liddell\" <sip:alice@wonderland.com>;foo=bar"
    // "contact: \"Alice Liddell\" <sip:alice@wonderland.com;foo=bar>"
    // "M: \"Alice Liddell\" <sip:alice@wonderland.com?foo=bar>"
    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com>;foo"
    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com;foo>"
    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com?foo>"
    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo=bar"
    // "Contact: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo"
    // "Contact: sip:alice@wonderland.com sip:hatter@wonderland.com"
    // "Contact: \"Alice Liddell\" <sips:alice@wonderland.com> \"Madison Hatter\" <sip:hatter@wonderland.com>"
    // "Contact: <sips:alice@wonderland.com> \"Madison Hatter\" <sip:hatter@wonderland.com>"
    // "Contact: \"Alice Liddell\" <sips:alice@wonderland.com> \"Madison Hatter\" <sip:hatter@wonderland.com>    sip:kat@cheshire.gov.uk"
    // "Contact: \"Alice Liddell\" <sips:alice@wonderland.com>;foo=bar \"Madison Hatter\" <sip:hatter@wonderland.com>    sip:kat@cheshire.gov.uk"
    // "Contact: \"Alice Liddell\" <sips:alice@wonderland.com> \"Madison Hatter\" <sip:hatter@wonderland.com>;foo=bar    sip:kat@cheshire.gov.uk"
    // "Contact: \"Alice Liddell\" <sips:alice@wonderland.com> \"Madison Hatter\" <sip:hatter@wonderland.com>    sip:kat@cheshire.gov.uk;foo=bar"

    #[test]
    fn test_parse() {
        let src = b"\"Mr. Watson\" <sip:watson@worcester.bell-telephone.com> \
        ;q=0.7; expires=3600\r\n";
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact {
            uri: SipUri::NameAddr(addr),
            q,
            expires,
            ..
        } => {
            assert_eq!(addr.display, Some("Mr. Watson".into()));
            assert_eq!(addr.uri.user.unwrap().user, "watson");
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

        let src = b"\"Mr. Watson\" <mailto:watson@bell-telephone.com> ;q=0.1\r\n";
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);

        assert!(contact.is_err());

        let src = b"sip:caller@u1.example.com\r\n";
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact {
            uri: SipUri::Uri(uri),
            ..
        } => {
            assert_eq!(uri.user.unwrap().user, "caller");
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
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact {
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
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact {
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
            assert_eq!(user.user, "thks.ashwin");
            assert_eq!(user.pass, Some("pass".into()));
        });
    }

    #[test]
    fn test_parse_host_port() {
        let src = b"sip:192.168.1.1:5060";
        let mut scanner = ParseCtx::new(src);
        let contact = Contact::parse(&mut scanner);
        let contact = contact.unwrap();

        assert_matches!(contact, Contact  {
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
