use crate::{
    bytes::Bytes, headers::SipHeader,
    message::auth::challenge::Challenge, parser::Result,
};

/// Consists of at least one challenge the 
/// authentication scheme(s) and parameters applicable
/// to the `Request-URI`.
pub struct WWWAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeader<'a> for WWWAuthenticate<'a> {
    const NAME: &'static str = "WWW-Authenticate";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let challenge = Challenge::parse(bytes)?;

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
        let mut bytes = Bytes::new(src);
        let www_auth = WWWAuthenticate::parse(&mut bytes);
        let www_auth = www_auth.unwrap();

        match www_auth.0 {
            Challenge::Digest(digest) => {
                assert_eq!(digest.realm, Some("atlanta.com"));
                assert_eq!(digest.algorithm, Some("MD5"));
                assert_eq!(digest.domain, Some("sip:boxesbybob.com"));
                assert_eq!(digest.qop, Some("auth"));
                assert_eq!(digest.nonce, Some("f84f1cec41e6cbe5aea9c8e88d359"));
                assert_eq!(digest.opaque, Some(""));
                assert_eq!(digest.stale, Some("FALSE"));
            }
            _ => unreachable!(),
        }
    }
}
