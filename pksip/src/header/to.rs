use std::fmt;
use std::str::{
    FromStr, {self},
};

use crate::error::Result;
use crate::header::{HeaderParser, TAG_PARAM};
use crate::macros::parse_header_param;
use crate::message::{Parameters, SipAddr, Uri};
use crate::parser::Parser;

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
///
/// # Examples
/// ```
/// # use pksip::{header::To};
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
/// let t = To::new(uri);
///
/// assert_eq!("To: <sip:alice@client.atlanta.example.com>", t.to_string());
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct To {
    uri: SipAddr,
    tag: Option<String>,
    params: Option<Parameters>,
}

impl FromStr for To {
    type Err = crate::error::Error;

    /// Parse a `To` header instance from a `&str`.
    fn from_str(s: &str) -> Result<Self> {
        Self::parse(&mut Parser::new(s.as_bytes()))
    }
}

impl To {
    /// Create a new `To` instance.
    pub fn new(uri: SipAddr) -> Self {
        Self {
            uri,
            tag: None,
            params: None,
        }
    }

    /// Get the SIP URI of the `To` header.
    pub fn sip_uri(&self) -> &SipAddr {
        &self.uri
    }

    /// Get the URI of the `To` header, if available.
    pub fn uri(&self) -> &Uri {
        self.uri.uri()
    }

    /// Get the display name of the `To` header, if available.
    pub fn display(&self) -> Option<&str> {
        self.uri.display()
    }

    /// Returns the tag parameter.
    pub fn tag(&self) -> &Option<String> {
        &self.tag
    }

    /// Set the tag parameter.
    pub fn set_tag(&mut self, tag: Option<String>) {
        self.tag = tag.map(|t| t.into());
    }
}

impl<'a> HeaderParser<'a> for To {
    const NAME: &'static str = "To";
    const SHORT_NAME: &'static str = "t";

    /*
     * To        =  ( "To" / "t" ) HCOLON ( name-addr
     *              / addr-spec ) *( SEMI to-param )
     * to-param  =  tag-param / generic-param
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let uri = parser.parse_sip_addr(false)?;
        let mut tag: Option<String> = None;
        let params = parse_header_param!(parser, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

impl fmt::Display for To {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", To::NAME, self.uri)?;
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
    // ToHeader inputs

    // "To: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "To : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "To  : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "To\t: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "To:\n  \"Alice Liddell\" \n\t<sip:alice@wonderland.com>"
    // "t: Alice <sip:alice@wonderland.com>"
    // "To: Alice sip:alice@wonderland.com>"
    // "To:"
    // "To: "
    // "To:\t"
    // "To: foo"
    // "To: foo bar"
    // "To: \"Alice\" sip:alice@wonderland.com>"
    // "To: \"<Alice>\" sip:alice@wonderland.com>"
    // "To: \"sip:alice@wonderland.com\""
    // "To: \"sip:alice@wonderland.com\"
    // <sip:alice@wonderland.com>" "T: \"<sip:alice@
    // wonderland.com>\"  <sip:alice@wonderland.com>"
    // "To: \"<sip: alice@wonderland.com>\"
    // <sip:alice@wonderland.com>" "To: \"Alice Liddell\"
    // <sip:alice@wonderland.com>;foo=bar" "To: sip:alice@
    // wonderland.com;foo=bar" "To: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo=bar>" "To: \"Alice
    // Liddell\" <sip:alice@wonderland.com?foo=bar>"
    // "to: \"Alice Liddell\" <sip:alice@wonderland.com>;foo"
    // "TO: \"Alice Liddell\" <sip:alice@wonderland.com;foo>"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com?foo>"
    // "To: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo?foo=bar>;foo=bar"
    // "To: \"Alice Liddell\"
    // <sip:alice@wonderland.com;foo?foo=bar>;foo"
    // "To: sip:alice@wonderland.com sip:hatter@wonderland.com"
    // "To: *"
    // "To: <*>"
    // "To: \"Alice Liddell\"<sip:alice@wonderland.com>"
    // "To: Alice Liddell <sip:alice@wonderland.com>"
    // "To: Alice Liddell<sip:alice@wonderland.com>"
    // "To: Alice<sip:alice@wonderland.com>"
    use super::*;
    use crate::message::{DomainName, Host, HostPort, Scheme};

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>;tag=a6c85cf\r\n";
        let mut scanner = Parser::new(src);
        let to = To::parse(&mut scanner);
        let to = to.unwrap();

        match to {
            To {
                uri: SipAddr::NameAddr(addr),
                tag,
                ..
            } => {
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(addr.display, Some("Bob".into()));
                assert_eq!(addr.uri.user.unwrap().user.as_ref(), "bob");
                assert_eq!(
                    addr.uri.host_port,
                    HostPort {
                        host: Host::DomainName(DomainName::new("biloxi.com")),
                        port: None,
                    }
                );
                assert_eq!(tag, Some("a6c85cf".into()));
            }
            _ => unreachable!(),
        }
    }
}
