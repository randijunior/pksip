use crate::{
    bytes::Bytes,
    macros::{parse_auth_param, read_while, space},
    parser::{self, is_token, Result},
    uri::Params,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct DigestCredential<'a> {
    pub realm: Option<&'a str>,
    pub username: Option<&'a str>,
    pub nonce: Option<&'a str>,
    pub uri: Option<&'a str>,
    pub response: Option<&'a str>,
    pub algorithm: Option<&'a str>,
    pub cnonce: Option<&'a str>,
    pub opaque: Option<&'a str>,
    pub qop: Option<&'a str>,
    pub nc: Option<&'a str>,
    pub param: Params<'a>,
}

impl<'a> DigestCredential<'a> {
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(bytes);
            match parser::parse_token(bytes) {
                "realm" => digest.realm = parse_auth_param!(bytes),
                "username" => digest.username = parse_auth_param!(bytes),
                "nonce" => digest.nonce = parse_auth_param!(bytes),
                "uri" => digest.uri = parse_auth_param!(bytes),
                "response" => digest.response = parse_auth_param!(bytes),
                "algorithm" => digest.algorithm = parse_auth_param!(bytes),
                "cnonce" => digest.cnonce = parse_auth_param!(bytes),
                "opaque" => digest.opaque = parse_auth_param!(bytes),
                "qop" => digest.qop = parse_auth_param!(bytes),
                "nc" => digest.nc = parse_auth_param!(bytes),
                other => {
                    digest.param.set(other, parse_auth_param!(bytes));
                }
            };

            if let Some(&b',') = bytes.peek() {
                bytes.next();
            } else {
                break;
            }
        }

        Ok(digest)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Credential<'a> {
    Digest(DigestCredential<'a>),
    Other { scheme: &'a str, param: Params<'a> },
}

/*
Authorization     =  "Authorization" HCOLON credentials
credentials       =  ("Digest" LWS digest-response)
                     / other-response
digest-response   =  dig-resp *(COMMA dig-resp)
dig-resp          =  username / realm / nonce / digest-uri
                      / dresponse / algorithm / cnonce
                      / opaque / message-qop
                      / nonce-count / auth-param
username          =  "username" EQUAL username-value
username-value    =  quoted-string
digest-uri        =  "uri" EQUAL LDQUOT digest-uri-value RDQUOT
digest-uri-value  =  rquest-uri ; Equal to request-uri as specified
                     by HTTP/1.1
message-qop       =  "qop" EQUAL qop-value

cnonce            =  "cnonce" EQUAL cnonce-value
cnonce-value      =  nonce-value
nonce-count       =  "nc" EQUAL nc-value
nc-value          =  8LHEX
dresponse         =  "response" EQUAL request-digest
request-digest    =  LDQUOT 32LHEX RDQUOT
auth-param        =  auth-param-name EQUAL
                     ( token / quoted-string )
auth-param-name   =  token
other-response    =  auth-scheme LWS auth-param
                     *(COMMA auth-param)
auth-scheme       =  token

*/
#[derive(Debug, PartialEq, Eq)]
pub struct Authorization<'a>(Credential<'a>);

impl<'a> Authorization<'a> {
    pub fn credential(&self) -> &Credential<'a> {
        &self.0
    }
}

impl<'a> SipHeaderParser<'a> for Authorization<'a> {
    const NAME: &'static [u8] = b"Authorization";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let credential = Self::parse_auth_credential(bytes)?;

        Ok(Authorization(credential))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest username=\"Alice\", realm=\"atlanta.com\", \
        nonce=\"84a4cc6f3082121f32b42a2187831a9e\",\
        response=\"7587245234b3434cc3412213e5f113a5432\"\r\n";
        let mut bytes = Bytes::new(src);
        let auth = Authorization::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        let cred = auth.credential();
        assert_matches!(cred, Credential::Digest(digest_credential) => {
            assert_eq!(digest_credential.username, Some("Alice"));
            assert_eq!(digest_credential.realm, Some("atlanta.com"));
            assert_eq!(digest_credential.nonce, Some("84a4cc6f3082121f32b42a2187831a9e"));
            assert_eq!(digest_credential.response, Some("7587245234b3434cc3412213e5f113a5432"));
        });
    }
}
