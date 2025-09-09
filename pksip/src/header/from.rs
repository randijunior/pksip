use core::fmt;
use std::str::{self, FromStr};
use std::sync::Arc;

use crate::error::Result;
use crate::header::{HeaderParser, TAG_PARAM};
use crate::macros::parse_header_param;
use crate::message::{Parameters, SipAddr, Uri};
use crate::parser::Parser;

/// The `From` SIP header.
///
/// Indicates the initiator of the request.
///
/// # Examples
/// ```
/// # use pksip::{header::From};
/// # use pksip::message::{HostPort, Host, UserInfo, UriBuilder, SipAddr, NameAddr};
/// let uri = SipAddr::NameAddr(NameAddr {
///     display: None,
///     uri: UriBuilder::new()
///         .with_user(UserInfo {
///             user: "alice".into(),
///             pass: None,
///         })
///         .with_host(HostPort::from(Host::DomainName(
///             "client.atlanta.example.com".into(),
///         )))
///         .build(),
/// });
///
/// let f = From::new(uri);
///
/// assert_eq!(
///     "From: <sip:alice@client.atlanta.example.com>",
///     f.to_string()
/// );
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct From {
    uri: SipAddr,
    tag: Option<Arc<str>>,
    params: Option<Parameters>,
}

impl FromStr for From {
    type Err = crate::error::Error;

    /// Parse a `From` header instance from a `&str`.
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(&mut Parser::new(s.as_bytes()))
    }
}

impl From {
    /// Create a new `From` instance.
    pub fn new(uri: SipAddr) -> Self {
        Self {
            uri,
            tag: None,
            params: None,
        }
    }

    /// Get the URI of the `From` header, if available.
    pub fn uri(&self) -> &Uri {
        self.uri.uri()
    }

    /// Get the display name of the `To` header, if available.
    pub fn display(&self) -> Option<&str> {
        self.uri.display()
    }

    /// Returns the tag parameter.
    pub fn tag(&self) -> &Option<Arc<str>> {
        &self.tag
    }
}

impl<'a> HeaderParser<'a> for From {
    const NAME: &'static str = "From";
    const SHORT_NAME: &'static str = "f";

    /*
     * From        =  ( "From" / "f" ) HCOLON from-spec
     * from-spec   =  ( name-addr / addr-spec )
     *                *( SEMI from-param )
     * from-param  =  tag-param / generic-param
     * tag-param   =  "tag" EQUAL token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let uri = parser.parse_sip_addr(false)?;
        let mut tag = None;
        let params = parse_header_param!(parser, TAG_PARAM = tag);

        Ok(From { tag, uri, params })
    }
}

impl fmt::Display for From {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.uri {
            SipAddr::Uri(uri) => write!(f, "{}: {}", From::NAME, uri)?,
            SipAddr::NameAddr(name_addr) => write!(f, "{}: {}", From::NAME, name_addr)?,
        }
        if let Some(tag) = &self.tag {
            write!(f, ";tag={}", tag)?;
        }
        if let Some(params) = &self.params {
            write!(f, "{}", params)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{DisplayName, DomainName, Host, HostPort, Scheme};

    // FromHeader inputs

    // "From: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From   : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From\t: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From:\n  \"Alice Liddell\"
    // \n\t<sip:alice@wonderland.com>" "f: Alice <sip:alice@
    // wonderland.com>" "From: Alice
    // sip:alice@wonderland.com>" "From:"
    // "From: "
    // "From:\t"
    // "From: foo"
    // "From: foo bar"
    // "From: \"Alice\" sip:alice@wonderland.com>"
    // "From: \"<Alice>\" sip:alice@wonderland.com>"
    // "From: \"sip:alice@wonderland.com\""
    // "From: \"sip:alice@wonderland.com\"
    // <sip:alice@wonderland.com>" "From: \"<sip:alice@
    // wonderland.com>\"  <sip:alice@wonderland.com>" "From:
    // \"<sip: alice@wonderland.com>\"
    // <sip:alice@wonderland.com>" "FrOm: \"Alice Liddell\"
    // <sip:alice@wonderland.com>;foo=bar" "FrOm: sip:alice@
    // wonderland.com;foo=bar" "from: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo=bar>" "F: \"Alice
    // Liddell\" <sip:alice@wonderland.com?foo=bar>"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com>;foo"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com;foo>"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com?foo>"
    // "From: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo?foo=bar>;foo=bar"
    // "From: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo?foo=bar>;foo"
    // "From: sip:alice@wonderland.com
    // sip:hatter@wonderland.com" "From: *"
    // "From: <*>"

    #[test]
    fn test_parse() {
        let src = b"\"A. G. Bell\" <sip:agb@bell-telephone.com> ;tag=a48s\r\n";
        let mut scanner = Parser::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipAddr::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some(DisplayName::new("A. G. Bell")));
            assert_eq!(addr.uri.user.unwrap().user.as_ref(), "agb");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName(DomainName::new("bell-telephone.com")),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("a48s".into()));
        });

        let src = b"sip:+12125551212@server.phone2net.com;tag=887s\r\n";
        let mut scanner = Parser::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipAddr::Uri(uri),
            tag,
            ..
        } => {
            assert_eq!(uri.user.unwrap().user.as_ref(), "+12125551212");
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::DomainName(DomainName::new("server.phone2net.com")),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("887s".into()));
        });

        let src = b"Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8\r\n";
        let mut scanner = Parser::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipAddr::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some(DisplayName::new("Anonymous")));
            assert_eq!(addr.uri.user.unwrap().user.as_ref(), "c8oqz84zk7z");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName(DomainName::new("privacy.org")),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("hyh8".into()));
         });
    }
}
