use std::fmt;

use crate::{error::Result, headers::SipHeaderParse, message::auth::Challenge, parser::ParseCtx};

/// The `WWW-Authenticate` SIP header.
///
/// Consists of at least one challenge the
/// authentication scheme(s) and parameters applicable
/// to the `Request-URI`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WWWAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeaderParse<'a> for WWWAuthenticate<'a> {
    const NAME: &'static str = "WWW-Authenticate";
    /*
     * WWW-Authenticate  =  "WWW-Authenticate" HCOLON challenge
     *
     * extension-header  =  header-name HCOLON header-value
     * header-name       =  token
     * header-value      =  *(TEXT-UTF8char / UTF8-CONT / LWS)
     * message-body  =  *OCTET
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let challenge = parser.parse_auth_challenge()?;

        Ok(WWWAuthenticate(challenge))
    }
}

impl fmt::Display for WWWAuthenticate<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", WWWAuthenticate::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::message::auth::DigestChallenge;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest realm=\"atlanta.com\",\
        domain=\"sip:boxesbybob.com\", qop=\"auth\",\
        nonce=\"f84f1cec41e6cbe5aea9c8e88d359\",\
        opaque=\"\", stale=FALSE, algorithm=MD5";
        let mut scanner = ParseCtx::new(src);
        let www_auth = WWWAuthenticate::parse(&mut scanner);
        let www_auth = www_auth.unwrap();

        assert_matches!(www_auth.0, Challenge::Digest (DigestChallenge { realm, domain, nonce, opaque, stale, algorithm, qop, .. }) => {
            assert_eq!(realm, Some("atlanta.com".into()));
            assert_eq!(algorithm, Some("MD5".into()));
            assert_eq!(domain, Some("sip:boxesbybob.com".into()));
            assert_eq!(qop, Some("auth".into()));
            assert_eq!(nonce, Some("f84f1cec41e6cbe5aea9c8e88d359".into()));
            assert_eq!(opaque, Some("".into()));
            assert_eq!(stale, Some("FALSE".into()));
        });
    }
}
