use crate::{
    macros::{parse_auth_param, read_while, sip_parse_error, space},
    parser::{is_token, Result},
    scanner::Scanner,
    uri::Params,
};

use super::SipHeaderParser;

use std::str;

pub struct AuthenticationInfo<'a> {
    nextnonce: Option<&'a str>,
    qop: Option<&'a str>,
    rspauth: Option<&'a str>,
    cnonce: Option<&'a str>,
    nc: Option<&'a str>,
}

impl<'a> SipHeaderParser<'a> for AuthenticationInfo<'a> {
    const NAME: &'static [u8] = b"Authentication-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut nextnonce: Option<&'a str> = None;
        let mut rspauth: Option<&'a str> = None;
        let mut qop: Option<&'a str> = None;
        let mut cnonce: Option<&'a str> = None;
        let mut nc: Option<&'a str> = None;

        macro_rules! parse {
            () => {
                space!(scanner);
                match read_while!(scanner, is_token) {
                    b"nextnonce" => nextnonce = parse_auth_param!(scanner),
                    b"qop" => qop = parse_auth_param!(scanner),
                    b"rspauth" => rspauth = parse_auth_param!(scanner),
                    b"cnonce" => cnonce = parse_auth_param!(scanner),
                    b"nc" => nc = parse_auth_param!(scanner),
                    _ => sip_parse_error!("Can't parse Authentication-Info")?,
                };
            };
        }

        parse!();
        while let Some(b',') = scanner.peek() {
            scanner.next();
            parse!();
        }

        Ok(AuthenticationInfo {
            nextnonce,
            qop,
            rspauth,
            cnonce,
            nc,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"nextnonce=\"47364c23432d2e131a5fb210812c\"\r\n";
        let mut scanner = Scanner::new(src);
        let auth_info = AuthenticationInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(auth_info.nextnonce, Some("47364c23432d2e131a5fb210812c"));

        let src = b"nextnonce=\"5ccc069c403ebaf9f0171e9517f40e41\", \
        cnonce=\"0a4f113b\", nc=00000001, \
        qop=\"auth\", \
        rspauth=\"6629fae49393a05397450978507c4ef1\"\r\n";
        let mut scanner = Scanner::new(src);
        let auth_info = AuthenticationInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(
            auth_info.nextnonce,
            Some("5ccc069c403ebaf9f0171e9517f40e41")
        );
        assert_eq!(auth_info.cnonce, Some("0a4f113b"));
        assert_eq!(
            auth_info.nextnonce,
            Some("5ccc069c403ebaf9f0171e9517f40e41")
        );
        assert_eq!(auth_info.nc, Some("00000001"));
        assert_eq!(auth_info.qop, Some("auth"));
        assert_eq!(auth_info.rspauth, Some("6629fae49393a05397450978507c4ef1"));
    }
}
