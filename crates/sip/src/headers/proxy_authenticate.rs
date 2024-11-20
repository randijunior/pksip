use scanner::Scanner;

use crate::{auth::Challenge, parser::Result};

use crate::headers::SipHeader;

/// The `Proxy-Authenticate` SIP header.
///
/// The authentication requirements from a proxy server to a client.
pub struct ProxyAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeader<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static str = "Proxy-Authenticate";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let challenge = Challenge::parse(scanner)?;

        Ok(ProxyAuthenticate(challenge))
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
        let mut scanner = Scanner::new(src);
        let proxy_auth = ProxyAuthenticate::parse(&mut scanner).unwrap();

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
