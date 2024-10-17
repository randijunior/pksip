use crate::{headers::SipHeaderParser, parser::Result, scanner::Scanner};

use super::proxy_authenticate::Challenge;

#[derive(Debug, PartialEq, Eq)]
pub struct WWWAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeaderParser<'a> for WWWAuthenticate<'a> {
    const NAME: &'static [u8] = b"WWW-Authenticate";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let challenge = Self::parse_auth_challenge(scanner)?;

        Ok(WWWAuthenticate(challenge))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest realm=\"atlanta.com\",\
        domain=\"sip:boxesbybob.com\", qop=\"auth\",\
        nonce=\"f84f1cec41e6cbe5aea9c8e88d359\",\
        opaque=\"\", stale=FALSE, algorithm=MD5";
        let mut scanner = Scanner::new(src);
        let www_auth = WWWAuthenticate::parse(&mut scanner);
        let www_auth = www_auth.unwrap();

        assert_matches!(www_auth.0, Challenge::Digest(digest) => {
            assert_eq!(digest.realm, Some("atlanta.com"));
            assert_eq!(digest.algorithm, Some("MD5"));
            assert_eq!(digest.domain, Some("sip:boxesbybob.com"));
            assert_eq!(digest.qop, Some("auth"));
            assert_eq!(digest.nonce, Some("f84f1cec41e6cbe5aea9c8e88d359"));
            assert_eq!(digest.opaque, Some(""));
            assert_eq!(digest.stale, Some("FALSE"));
        });
    }
}
