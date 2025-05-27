use crate::{
    error::Result,
    headers::TAG_PARAM,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::ParseCtx,
};

use crate::headers::SipHeaderParse;

use std::{
    fmt,
    str::{self},
};

/// The `To` SIP header.
///
/// Specifies the logical recipient of the request.
///
/// # Examples
/// ```
/// # use pksip::{headers::To};
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
/// let t = To::new(uri);
///
/// assert_eq!(
///     "To: <sip:alice@client.atlanta.example.com>",
///     t.to_string()
/// );
///
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct To<'a> {
    uri: SipUri<'a>,
    tag: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> To<'a> {
    /// Create a new `To` instance.
    pub fn new(uri: SipUri<'a>) -> Self {
        Self {
            uri,
            tag: None,
            params: None,
        }
    }

    /// Get the URI of the `To` header.
    pub fn uri(&self) -> &SipUri {
        &self.uri
    }
    /// Returns the tag parameter.
    pub fn tag(&self) -> Option<&'a str> {
        self.tag
    }

    /// Set the tag parameter.
    pub fn set_tag(&mut self, tag: Option<&'a str>) {
        self.tag = tag;
    }
}

impl<'a> SipHeaderParse<'a> for To<'a> {
    const NAME: &'static str = "To";
    const SHORT_NAME: &'static str = "t";
    /*
     * To        =  ( "To" / "t" ) HCOLON ( name-addr
     *              / addr-spec ) *( SEMI to-param )
     * to-param  =  tag-param / generic-param
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let uri = parser.parse_sip_uri(false)?;
        let mut tag = None;
        let params = parse_header_param!(parser, TAG_PARAM = tag);

        Ok(To { tag, uri, params })
    }
}

impl fmt::Display for To<'_> {
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
    use crate::message::{Host, HostPort, Scheme};

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
    // "To: \"sip:alice@wonderland.com\"  <sip:alice@wonderland.com>"
    // "T: \"<sip:alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "To: \"<sip: alice@wonderland.com>\"  <sip:alice@wonderland.com>"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com>;foo=bar"
    // "To: sip:alice@wonderland.com;foo=bar"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com;foo=bar>"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com?foo=bar>"
    // "to: \"Alice Liddell\" <sip:alice@wonderland.com>;foo"
    // "TO: \"Alice Liddell\" <sip:alice@wonderland.com;foo>"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com?foo>"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo=bar"
    // "To: \"Alice Liddell\" <sip:alice@wonderland.com;foo?foo=bar>;foo"
    // "To: sip:alice@wonderland.com sip:hatter@wonderland.com"
    // "To: *"
    // "To: <*>"
    // "To: \"Alice Liddell\"<sip:alice@wonderland.com>"
    // "To: Alice Liddell <sip:alice@wonderland.com>"
    // "To: Alice Liddell<sip:alice@wonderland.com>"
    // "To: Alice<sip:alice@wonderland.com>"

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>;tag=a6c85cf\r\n";
        let mut scanner = ParseCtx::new(src);
        let to = To::parse(&mut scanner);
        let to = to.unwrap();

        match to {
            To {
                uri: SipUri::NameAddr(addr),
                tag,
                ..
            } => {
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(addr.display, Some("Bob".into()));
                assert_eq!(addr.uri.user.unwrap().user, "bob");
                assert_eq!(
                    addr.uri.host_port,
                    HostPort {
                        host: Host::DomainName("biloxi.com".into()),
                        port: None,
                    }
                );
                assert_eq!(tag, Some("a6c85cf".into()));
            }
            _ => unreachable!(),
        }
    }
}
