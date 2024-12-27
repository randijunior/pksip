use std::fmt;

use reader::Reader;

use crate::{auth::Challenge, parser::Result};

use crate::headers::SipHeader;

/// The `Proxy-Authenticate` SIP header.
///
/// The authentication requirements from a proxy server to a client.
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeader<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static str = "Proxy-Authenticate";
    /*
     * Proxy-Authenticate  =  "Proxy-Authenticate" HCOLON challenge
     * challenge           =  ("Digest" LWS digest-cln *(COMMA digest-cln))
     *                        / other-challenge
     * other-challenge     =  auth-scheme LWS auth-param
     *                        *(COMMA auth-param)
     * digest-cln          =  realm / domain / nonce
     *                         / opaque / stale / algorithm
     *                         / qop-options / auth-param
     * realm               =  "realm" EQUAL realm-value
     * realm-value         =  quoted-string
     * domain              =  "domain" EQUAL LDQUOT URI
     *                        *( 1*SP URI ) RDQUOT
     * URI                 =  absoluteURI / abs-path
     * nonce               =  "nonce" EQUAL nonce-value
     * nonce-value         =  quoted-string
     * opaque              =  "opaque" EQUAL quoted-string
     * stale               =  "stale" EQUAL ( "true" / "false" )
     * algorithm           =  "algorithm" EQUAL ( "MD5" / "MD5-sess"
     *                        / token )
     * qop-options         =  "qop" EQUAL LDQUOT qop-value
     *                        *("," qop-value) RDQUOT
     * qop-value           =  "auth" / "auth-int" / token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let challenge = Challenge::parse(reader)?;

        Ok(ProxyAuthenticate(challenge))
    }
}

impl fmt::Display for ProxyAuthenticate<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest realm=\"atlanta.com\", \
        domain=\"sip:ss1.carrier.com\", qop=\"auth\", \
        nonce=\"f84f1cec41e6cbe5aea9c8e88d359\", \
        opaque=\"\", stale=FALSE, algorithm=MD5\r\n";
        let mut reader = Reader::new(src);
        let proxy_auth = ProxyAuthenticate::parse(&mut reader).unwrap();

        assert_matches!(proxy_auth.0, Challenge::Digest { realm, domain, nonce, opaque, stale, algorithm, qop, .. } => {
            assert_eq!(realm, Some("atlanta.com"));
            assert_eq!(algorithm, Some("MD5"));
            assert_eq!(domain, Some("sip:ss1.carrier.com"));
            assert_eq!(qop, Some("auth"));
            assert_eq!(nonce, Some("f84f1cec41e6cbe5aea9c8e88d359"));
            assert_eq!(opaque, Some(""));
            assert_eq!(stale, Some("FALSE"));
        });
    }
}
