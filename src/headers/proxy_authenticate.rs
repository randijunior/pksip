use crate::{
    byte_reader::ByteReader,
    macros::{parse_auth_param, read_while, space},
    parser::{is_token, Result},
    uri::Params,
};

use super::SipHeaderParser;

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
#[derive(Debug, Default)]
pub struct DigestChallenge<'a> {
    realm: &'a str,
    domain: Option<&'a str>,
    nonce: &'a str,
    opaque: Option<&'a str>,
    stale: Option<&'a str>,
    algorithm: &'a str,
    qop: &'a str,
    param: Params<'a>,
}

impl<'a> DigestChallenge<'a> {
    pub(crate) fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(reader);
            match read_while!(reader, is_token) {
                b"realm" => digest.realm = parse_auth_param!(reader).unwrap_or(""),
                b"nonce" => digest.nonce = parse_auth_param!(reader).unwrap_or(""),
                b"domain" => digest.domain = parse_auth_param!(reader),
                b"algorithm" => digest.algorithm = parse_auth_param!(reader).unwrap_or(""),
                b"opaque" => digest.opaque = parse_auth_param!(reader),
                b"qop" => digest.qop = parse_auth_param!(reader).unwrap_or(""),
                other => {
                    digest.param.set(
                        unsafe { std::str::from_utf8_unchecked(other) },
                        parse_auth_param!(reader),
                    );
                }
            };

            if let Some(&b',') = reader.peek() {
                reader.next();
            } else {
                break;
            }
        }

        Ok(digest)
    }
}

pub enum Challenge<'a> {
    Digest(DigestChallenge<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}
pub struct ProxyAuthenticate<'a> {
    challenge: Challenge<'a>,
}

impl<'a> SipHeaderParser<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static [u8] = b"Proxy-Authenticate";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let challenge = Self::parse_auth_challenge(reader)?;

        Ok(ProxyAuthenticate { challenge })
    }
}
