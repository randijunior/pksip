use crate::{
    macros::{parse_auth_param, read_while, space},
    parser::{self, is_token, Result},
    scanner::Scanner,
    uri::Params,
};

use crate::headers::SipHeaderParser;

/*
    pj_str_t    realm;          /**< Realm for the challenge.   */
    pjsip_param other_param;    /**< Other parameters.          */
    pj_str_t    domain;         /**< Domain.                    */
    pj_str_t    nonce;          /**< Nonce challenge.           */
    pj_str_t    opaque;         /**< Opaque value.              */
    int         stale;          /**< Stale parameter.           */
    pj_str_t    algorithm;      /**< Algorithm parameter.       */
    pj_str_t    qop;


*/
#[derive(Debug, Default, PartialEq, Eq)]
pub struct DigestChallenge<'a> {
    pub realm: Option<&'a str>,
    pub domain: Option<&'a str>,
    pub nonce: Option<&'a str>,
    pub opaque: Option<&'a str>,
    pub stale: Option<&'a str>,
    pub algorithm: Option<&'a str>,
    pub qop: Option<&'a str>,
    pub param: Params<'a>,
}

impl<'a> DigestChallenge<'a> {
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(scanner);
            match parser::parse_token(scanner) {
                "realm" => digest.realm = parse_auth_param!(scanner),
                "nonce" => digest.nonce = parse_auth_param!(scanner),
                "domain" => digest.domain = parse_auth_param!(scanner),
                "algorithm" => digest.algorithm = parse_auth_param!(scanner),
                "opaque" => digest.opaque = parse_auth_param!(scanner),
                "qop" => digest.qop = parse_auth_param!(scanner),
                "stale" => digest.stale = parse_auth_param!(scanner),
                other => {
                    digest.param.set(other, parse_auth_param!(scanner));
                }
            };

            if let Some(&b',') = scanner.peek() {
                scanner.next();
            } else {
                break;
            }
        }

        Ok(digest)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Challenge<'a> {
    Digest(DigestChallenge<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}
#[derive(Debug, PartialEq, Eq)]
pub struct ProxyAuthenticate<'a>(Challenge<'a>);

impl<'a> SipHeaderParser<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static [u8] = b"Proxy-Authenticate";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let challenge = Self::parse_auth_challenge(scanner)?;

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

        assert_matches!(proxy_auth.0, Challenge::Digest(digest) => {
            assert_eq!(digest.realm, Some("atlanta.com"));
            assert_eq!(digest.algorithm, Some("MD5"));
            assert_eq!(digest.domain, Some("sip:ss1.carrier.com"));
            assert_eq!(digest.qop, Some("auth"));
            assert_eq!(digest.nonce, Some("f84f1cec41e6cbe5aea9c8e88d359"));
            assert_eq!(digest.opaque, Some(""));
            assert_eq!(digest.stale, Some("FALSE"));
        });
    }
}
