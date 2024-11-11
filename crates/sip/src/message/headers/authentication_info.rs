use crate::{
    bytes::Bytes,
    macros::{parse_comma_separated, sip_parse_error},
    message::auth::{CNONCE, NC, NEXTNONCE, QOP, RSPAUTH},
    parser::Result,
};

use crate::headers::SipHeader;

use std::str;

/// The `Authentication-Info` SIP header.
///
/// Provides additional authentication information.
#[derive(Debug, Default)]
pub struct AuthenticationInfo<'a> {
    nextnonce: Option<&'a str>,
    qop: Option<&'a str>,
    rspauth: Option<&'a str>,
    cnonce: Option<&'a str>,
    nc: Option<&'a str>,
}

impl<'a> SipHeader<'a> for AuthenticationInfo<'a> {
    const NAME: &'static str = "Authentication-Info";

    fn parse(bytes: &mut Bytes<'a>) -> Result<AuthenticationInfo<'a>> {
        let mut auth_info = AuthenticationInfo::default();

        parse_comma_separated!(bytes => {
            let (name, value) = super::parse_header_param(bytes)?;
            match name {
                NEXTNONCE => auth_info.nextnonce = value,
                QOP => auth_info.qop = value,
                RSPAUTH => auth_info.rspauth = value,
                CNONCE => auth_info.cnonce = value,
                NC => auth_info.nc = value,
                _ => sip_parse_error!("Can't parse Authentication-Info")?,
            };
        });

        Ok(auth_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"nextnonce=\"47364c23432d2e131a5fb210812c\"\r\n";
        let mut bytes = Bytes::new(src);
        let auth_info = AuthenticationInfo::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
        assert_eq!(auth_info.nextnonce, Some("47364c23432d2e131a5fb210812c"));

        let src = b"nextnonce=\"5ccc069c403ebaf9f0171e9517f40e41\", \
        cnonce=\"0a4f113b\", nc=00000001, \
        qop=\"auth\", \
        rspauth=\"6629fae49393a05397450978507c4ef1\"\r\n";
        let mut bytes = Bytes::new(src);
        let auth_info = AuthenticationInfo::parse(&mut bytes).unwrap();

        assert_eq!(bytes.as_ref(), b"\r\n");
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
