use std::fmt;

use crate::{error::Result, headers::HeaderParser, message::Challenge, parser::Parser};

/// The `Proxy-Authenticate` SIP header.
///
/// The authentication requirements from a proxy server to a
/// client.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProxyAuthenticate(Challenge);

impl<'a> HeaderParser<'a> for ProxyAuthenticate {
    const NAME: &'static str = "Proxy-Authenticate";

    /*
     * Proxy-Authenticate  =  "Proxy-Authenticate" HCOLON
     * challenge challenge           =  ("Digest" LWS
     * digest-cln *(COMMA digest-cln))
     * / other-challenge other-challenge     =
     * auth-scheme LWS auth-param
     * *(COMMA auth-param) digest-cln          =  realm /
     * domain / nonce                         / opaque /
     * stale / algorithm                         /
     * qop-options / auth-param realm               =
     * "realm" EQUAL realm-value realm-value         =
     * quoted-string domain              =  "domain"
     * EQUAL LDQUOT URI                        *( 1*SP
     * URI ) RDQUOT URI                 =  absoluteURI /
     * abs-path nonce               =  "nonce" EQUAL
     * nonce-value nonce-value         =  quoted-string
     * opaque              =  "opaque" EQUAL quoted-string
     * stale               =  "stale" EQUAL ( "true" /
     * "false" ) algorithm           =  "algorithm" EQUAL
     * ( "MD5" / "MD5-sess"                        /
     * token ) qop-options         =  "qop" EQUAL LDQUOT
     * qop-value                        *("," qop-value)
     * RDQUOT qop-value           =  "auth" / "auth-int"
     * / token
     */
    fn parse(parser: &mut Parser<'a>) -> Result<Self> {
        let challenge = parser.parse_auth_challenge()?;

        Ok(ProxyAuthenticate(challenge))
    }
}

impl fmt::Display for ProxyAuthenticate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ProxyAuthenticate::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::DigestChallenge;

    #[test]
    fn test_parse() {
        let src = b"Digest realm=\"atlanta.com\", \
        domain=\"sip:ss1.carrier.com\", qop=\"auth\", \
        nonce=\"f84f1cec41e6cbe5aea9c8e88d359\", \
        opaque=\"\", stale=FALSE, algorithm=MD5\r\n";
        let mut scanner = Parser::new(src);
        let proxy_auth = ProxyAuthenticate::parse(&mut scanner).unwrap();

        assert_matches!(proxy_auth.0, Challenge::Digest( DigestChallenge { realm, domain, nonce, opaque, stale, algorithm, qop, .. }) => {
            assert_eq!(realm, Some("\"atlanta.com\"".into()));
            assert_eq!(algorithm, Some("MD5".into()));
            assert_eq!(domain, Some("\"sip:ss1.carrier.com\"".into()));
            assert_eq!(qop, Some("\"auth\"".into()));
            assert_eq!(nonce, Some("\"f84f1cec41e6cbe5aea9c8e88d359\"".into()));
            assert_eq!(opaque, Some("\"\"".into()));
            assert_eq!(stale, Some("FALSE".into()));
        });
    }
}
