use crate::{
    byte_reader::ByteReader,
    macros::{parse_auth_param, parse_param, read_until_byte, read_while, sip_parse_error, space},
    parser::{is_token, Result},
    uri::Params,
};

use super::SipHeaderParser;

#[derive(Debug, Default)]
pub struct Digest<'a> {
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

impl<'a> Digest<'a> {
    pub fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut digest = Digest::default();
        loop {
            space!(reader);
            match read_while!(reader, is_token) {
                b"realm" => digest.realm = parse_auth_param!(reader),
                b"username" => digest.username = parse_auth_param!(reader),
                b"nonce" => digest.nonce = parse_auth_param!(reader),
                b"uri" => digest.uri = parse_auth_param!(reader),
                b"response" => digest.response = parse_auth_param!(reader),
                b"algorithm" => digest.algorithm = parse_auth_param!(reader),
                b"cnonce" => digest.cnonce = parse_auth_param!(reader),
                b"opaque" => digest.opaque = parse_auth_param!(reader),
                b"qop" => digest.qop = parse_auth_param!(reader),
                b"nc" => digest.nc = parse_auth_param!(reader),
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

pub enum Credential<'a> {
    Digest(Digest<'a>),
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

pub struct Authorization<'a>(Credential<'a>);

impl<'a> SipHeaderParser<'a> for Authorization<'a> {
    const NAME: &'static [u8] = b"Authorization";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let cred = Self::parse_auth_credential(reader)?;

        Ok(Authorization(cred))
    }
}
