use crate::{
    macros::{parse_auth_param, read_while, space},
    parser::{is_token, Result},
    scanner::Scanner,
    uri::Params,
};

use super::SipHeaderParser;

#[derive(Debug, Default, PartialEq)]
pub struct DigestCredential<'a> {
    realm: Option<&'a str>,
    username: Option<&'a str>,
    nonce: Option<&'a str>,
    uri: Option<&'a str>,
    response: Option<&'a str>,
    algorithm: Option<&'a str>,
    cnonce: Option<&'a str>,
    opaque: Option<&'a str>,
    qop: Option<&'a str>,
    nc: Option<&'a str>,
    param: Params<'a>,
}

impl<'a> DigestCredential<'a> {
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut digest = Self::default();
        loop {
            space!(scanner);
            match read_while!(scanner, is_token) {
                b"realm" => digest.realm = parse_auth_param!(scanner),
                b"username" => digest.username = parse_auth_param!(scanner),
                b"nonce" => digest.nonce = parse_auth_param!(scanner),
                b"uri" => digest.uri = parse_auth_param!(scanner),
                b"response" => digest.response = parse_auth_param!(scanner),
                b"algorithm" => digest.algorithm = parse_auth_param!(scanner),
                b"cnonce" => digest.cnonce = parse_auth_param!(scanner),
                b"opaque" => digest.opaque = parse_auth_param!(scanner),
                b"qop" => digest.qop = parse_auth_param!(scanner),
                b"nc" => digest.nc = parse_auth_param!(scanner),
                other => {
                    digest.param.set(
                        unsafe { std::str::from_utf8_unchecked(other) },
                        parse_auth_param!(scanner),
                    );
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
#[derive(Debug, PartialEq)]
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
#[derive(Debug)]
pub struct Authorization<'a>(Credential<'a>);

impl<'a> Authorization<'a> {
    pub fn credential(&self) -> &Credential<'a> {
        &self.0
    }
}

impl<'a> SipHeaderParser<'a> for Authorization<'a> {
    const NAME: &'static [u8] = b"Authorization";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let credential = Self::parse_auth_credential(scanner)?;

        Ok(Authorization(credential))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest username=\"Alice\", realm=\"atlanta.com\", nonce=\"84a4cc6f3082121f32b42a2187831a9e\",response=\"7587245234b3434cc3412213e5f113a5432\"\r\n";
        let mut scanner = Scanner::new(src);
        let auth = Authorization::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        let cred = auth.credential();
        assert_matches!(cred, Credential::Digest(digest_credential) => {
            assert_eq!(digest_credential.username, Some("Alice"));
            assert_eq!(digest_credential.realm, Some("atlanta.com"));
            assert_eq!(digest_credential.nonce, Some("84a4cc6f3082121f32b42a2187831a9e"));
            assert_eq!(digest_credential.response, Some("7587245234b3434cc3412213e5f113a5432"));
        });
    }
}
