use std::fmt;

use reader::Reader;

use crate::{auth::Credential, parser::Result};

use super::SipHeader;

/// The `Authorization` SIP header.
///
/// Contains authentication credentials of a `UA`.
#[derive(Debug, PartialEq, Eq)]
pub struct Authorization<'a>(Credential<'a>);

impl<'a> Authorization<'a> {
    pub fn credential(&self) -> &Credential<'a> {
        &self.0
    }
}

impl<'a> SipHeader<'a> for Authorization<'a> {
    const NAME: &'static str = "Authorization";
    /*
     * Authorization     =  "Authorization" HCOLON credentials
     * credentials       =  ("Digest" LWS digest-response)
     *			            / other-response
     * digest-response   =  dig-resp *(COMMA dig-resp)
     * dig-resp          =  username / realm / nonce / digest-uri
     *			            / dresponse / algorithm / cnonce
     *			            / opaque / message-qop
     *			            / nonce-count / auth-param
     * username          =  "username" EQUAL username-value
     * username-value    =  quoted-string
     * digest-uri        =  "uri" EQUAL LDQUOT digest-uri-value RDQUOT
     * digest-uri-value  =  rquest-uri ; Equal to request-uri as specified
     *			            by HTTP/1.1
     * message-qop       =  "qop" EQUAL qop-value
     * cnonce            =  "cnonce" EQUAL cnonce-value
     * cnonce-value      =  nonce-value
     * nonce-count       =  "nc" EQUAL nc-value
     * nc-value          =  8LHEX
     * dresponse         =  "response" EQUAL request-digest
     * request-digest    =  LDQUOT 32LHEX RDQUOT
     * auth-param        =  auth-param-name EQUAL
     * 			            ( token / quoted-string )
     * auth-param-name   =  token
     * other-response    =  auth-scheme LWS auth-param
     *			            *(COMMA auth-param)
     * auth-scheme       =  token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Authorization<'a>> {
        let credential = Credential::parse(reader)?;

        Ok(Authorization(credential))
    }
}

impl fmt::Display for Authorization<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
        let mut reader = Reader::new(src);
        let auth = Authorization::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_matches!(auth.0, Credential::Digest { username, realm, nonce, response, ..} => {
            assert_eq!(username, Some("Alice"));
            assert_eq!(realm, Some("atlanta.com"));
            assert_eq!(
                nonce,
                Some("84a4cc6f3082121f32b42a2187831a9e")
            );
            assert_eq!(
                response,
                Some("7587245234b3434cc3412213e5f113a5432")
            );
        });
    }
}
