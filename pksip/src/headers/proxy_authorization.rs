use std::fmt;

use crate::{error::Result, headers::SipHeaderParse, message::auth::Credential, parser::ParseCtx};

/// The `Proxy-Authorization` SIP header.
///
/// Consists of credentials containing the authentication
/// information of the user agent for the proxy.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ProxyAuthorization<'a>(Credential<'a>);

impl<'a> SipHeaderParse<'a> for ProxyAuthorization<'a> {
    const NAME: &'static str = "Proxy-Authorization";
    /*
     * Proxy-Authorization  =  "Proxy-Authorization" HCOLON credentials
     */
    fn parse(parser: &mut ParseCtx<'a>) -> Result<Self> {
        let credential = parser.parse_auth_credential()?;

        Ok(ProxyAuthorization(credential))
    }
}

impl fmt::Display for ProxyAuthorization<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", ProxyAuthorization::NAME, self.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::message::auth::DigestCredential;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest username=\"Alice\", realm=\"atlanta.com\", \
        nonce=\"c60f3082ee1212b402a21831ae\", \
        response=\"245f23415f11432b3434341c022\"\r\n";
        let mut scanner = ParseCtx::new(src);
        let proxy_auth = ProxyAuthorization::parse(&mut scanner).unwrap();

        assert_matches!(proxy_auth.0, Credential::Digest (DigestCredential { realm, username, nonce, response, .. }) => {
            assert_eq!(username, Some("Alice".into()));
            assert_eq!(realm, Some("atlanta.com".into()));
            assert_eq!(nonce, Some("c60f3082ee1212b402a21831ae".into()));
            assert_eq!(
                response,
                Some("245f23415f11432b3434341c022".into())
            );
        });
    }
}
