use crate::{byte_reader::ByteReader, uri::Params, parser::Result};

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

pub struct DigestChallenge<'a> {
    realm: &'a str,
    domain: Option<&'a str>,
    nonce: &'a str,
    opaque: Option<&'a str>,
    stale: i32,
    algorithm: &'a str,
    qop: &'a str,
    param: Params<'a>
}

pub enum Challenge<'a> {
    DigestChallenge(DigestChallenge<'a>)
}
pub struct ProxyAuthenticate<'a> {
    challenge: Challenge<'a>
}

impl<'a> SipHeaderParser<'a> for ProxyAuthenticate<'a> {
    const NAME: &'static [u8] = b"Proxy-Authenticate";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let cred = Self::parse_auth_credential(reader)?;

        todo!()
    }
}
