use crate::{bytes::Bytes, message::auth::digest::Credential, parser::Result};

use super::SipHeaderParser;

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

pub struct Authorization<'a>(Credential<'a>);

impl<'a> Authorization<'a> {
    pub fn credential(&self) -> &Credential<'a> {
        &self.0
    }
}

impl<'a> SipHeaderParser<'a> for Authorization<'a> {
    const NAME: &'static [u8] = b"Authorization";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let credential = Credential::parse(bytes)?;

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
        let digest_credential = match cred {
            Credential::Digest(digest_credential) => digest_credential,
            _ => unreachable!("The credential is digest!"),
        };

        assert_eq!(digest_credential.username, Some("Alice"));
        assert_eq!(digest_credential.realm, Some("atlanta.com"));
        assert_eq!(
            digest_credential.nonce,
            Some("84a4cc6f3082121f32b42a2187831a9e")
        );
        assert_eq!(
            digest_credential.response,
            Some("7587245234b3434cc3412213e5f113a5432")
        );
    }
}
