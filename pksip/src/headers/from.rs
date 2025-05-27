use crate::{
    error::Result,
    headers::TAG_PARAM,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::ParseCtx,
};

use crate::headers::SipHeaderParse;

use core::fmt;
use std::str::{self};

/// The `From` SIP header.
///
/// Indicates the initiator of the request.
///
/// # Examples
/// ```
/// # use pksip::{headers::From};
/// # use pksip::message::{HostPort, Host, UriUser, UriBuilder, SipUri, NameAddr};
/// let uri = SipUri::NameAddr(NameAddr {
///     display: None,
///     uri: UriBuilder::new()
///         .user(UriUser { user: "alice", pass: None })
///         .host(HostPort::from(Host::DomainName(
///             "client.atlanta.example.com".into(),
///         )))
///         .get(),
/// });
///
/// let f = From::new(uri);
///
/// assert_eq!(
///     "From: <sip:alice@client.atlanta.example.com>",
///     f.to_string()
/// );
///
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct From<'f> {
    uri: SipUri<'f>,
    tag: Option<&'f str>,
    params: Option<Params<'f>>,
}

impl<'f> From<'f> {
    /// Create a new `From` instance.
    pub fn new(uri: SipUri<'f>) -> Self {
        Self {
            uri,
            tag: None,
            params: None,
        }
    }
    /// Get the URI of the `From` header.
    pub fn uri(&self) -> &SipUri {
        &self.uri
    }
    /// Returns the tag parameter.
    pub fn tag(&self) -> Option<&'f str> {
        self.tag
    }
}

impl<'a> SipHeaderParse<'a> for From<'a> {
    const NAME: &'static str = "From";
    const SHORT_NAME: &'static str = "f";
    /*
     * From        =  ( "From" / "f" ) HCOLON from-spec
     * from-spec   =  ( name-addr / addr-spec )
     *                *( SEMI from-param )
     * from-param  =  tag-param / generic-param
     * tag-param   =  "tag" EQUAL token
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let uri = parser.parse_sip_uri(false)?;
        let mut tag = None;
        let params = parse_header_param!(parser, TAG_PARAM = tag);

        Ok(From { tag, uri, params })
    }
}

impl fmt::Display for From<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.uri {
            SipUri::Uri(uri) => write!(f, "{}: {}", From::NAME, uri)?,
            SipUri::NameAddr(name_addr) => write!(f, "{}: {}", From::NAME, name_addr)?,
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
    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    // FromHeader inputs

    // "From: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From   : \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From\t: \"Alice Liddell\" <sip:alice@wonderland.com>"
    // "From:\n  \"Alice Liddell\" \n\t<sip:alice@wonderland.com>"
    // "f: Alice <sip:alice@wonderland.com>"
    // "From: Alice sip:alice@wonderland.com>"
    // "From:"
    // "From: "
    // "From:\t"
    // "From: foo"
    // "From: foo bar"
    // "From: \"Alice\" sip:alice@wonderland.com>"
    // "From: \"<Alice>\" sip:alice@wonderland.com>"
    // "From: \"sip:alice@wonderland.com\""
    // "From: \"sip:alice@wonderland.com\"  <sip:alice@wonderland.com>"
    // "From: \"<sip:alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "From: \"<sip: alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "FrOm: \"Alice Liddell\" <sip:alice@wonderland.com>;foo=bar"
    // "FrOm: sip:alice@wonderland.com;foo=bar"
    // "from: \"Alice Liddell\" <sip:alice@wonderland.com;foo=bar>"
    // "F: \"Alice Liddell\" <sip:alice@wonderland.com?foo=bar>"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com>;foo"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com;foo>"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com?foo>"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo=bar"
    // "From: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo"
    // "From: sip:alice@wonderland.com sip:hatter@wonderland.com"
    // "From: *"
    // "From: <*>"

    #[test]
    fn test_parse() {
        let src = b"\"A. G. Bell\" <sip:agb@bell-telephone.com> ;tag=a48s\r\n";
        let mut scanner = ParseCtx::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipUri::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some("A. G. Bell".into()));
            assert_eq!(addr.uri.user.unwrap().user, "agb");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("bell-telephone.com".into()),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("a48s".into()));
        });

        let src = b"sip:+12125551212@server.phone2net.com;tag=887s\r\n";
        let mut scanner = ParseCtx::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipUri::Uri(uri),
            tag,
            ..
        } => {
            assert_eq!(uri.user.unwrap().user, "+12125551212");
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::DomainName("server.phone2net.com".into()),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("887s".into()));
        });

        let src = b"Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8\r\n";
        let mut scanner = ParseCtx::new(src);
        let from = From::parse(&mut scanner).unwrap();

        assert_matches!(from, From {
            uri: SipUri::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some("Anonymous".into()));
            assert_eq!(addr.uri.user.unwrap().user, "c8oqz84zk7z");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("privacy.org".into()),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("hyh8".into()));
         });
    }
}
