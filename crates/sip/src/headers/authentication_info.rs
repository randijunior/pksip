use reader::Reader;

use crate::{
    auth::{CNONCE, NC, NEXTNONCE, QOP, RSPAUTH},
    internal::Param,
    macros::{comma_sep, sip_parse_error},
    parser::{Result, SipParserError},
};

use crate::headers::SipHeader;

use std::{fmt, str};

/// The `Authentication-Info` SIP header.
///
/// Provides additional authentication information.
///
/// # Examples
///
/// ```
/// # use sip::headers::AuthenticationInfo;
/// let mut auth = AuthenticationInfo::default();
/// auth.set_nextnonce(Some("5ccc069c403ebaf9f0171e9517f40e41"));
///
/// assert_eq!("nextnonce=\"5ccc069c403ebaf9f0171e9517f40e41\"".as_bytes().try_into(), Ok(auth));
/// ```
#[derive(Debug, Default, PartialEq, Eq)]
pub struct AuthenticationInfo<'a> {
    nextnonce: Option<&'a str>,
    qop: Option<&'a str>,
    rspauth: Option<&'a str>,
    cnonce: Option<&'a str>,
    nc: Option<&'a str>,
}

impl<'a> AuthenticationInfo<'a> {
    pub fn set_nextnonce(&mut self, nextnonce: Option<&'a str>) {
        self.nextnonce = nextnonce;
    }
}

impl<'a> TryFrom<&'a [u8]> for AuthenticationInfo<'a> {
    type Error = SipParserError;
    
    fn try_from(value: &'a [u8]) -> Result<Self> {
        Self::from_bytes(value)
    }
}


impl<'a> SipHeader<'a> for AuthenticationInfo<'a> {
    const NAME: &'static str = "Authentication-Info";
    /*
     * Authentication-Info  =  "Authentication-Info" HCOLON ainfo
     *                          *(COMMA ainfo)
     * ainfo                =  nextnonce / message-qop
     *				            / response-auth / cnonce
     *				            / nonce-count
     * nextnonce            =  "nextnonce" EQUAL nonce-value
     * response-auth        =  "rspauth" EQUAL response-digest
     * response-digest      =  LDQUOT *LHEX RDQUOT
     *
     */
    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let mut auth_info = AuthenticationInfo::default();

        comma_sep!(reader => {
            let Param {name, value} = Param::parse(reader)?;
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

impl fmt::Display for AuthenticationInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(nextnonce) = self.nextnonce {
            write!(f, "nextnonce={}", nextnonce)?;
        }
        if let Some(qop) = self.qop {
            write!(f, ", qop={}", qop)?;
        }
        if let Some(rspauth) = self.rspauth {
            write!(f, ", rspauth={}", rspauth)?;
        }
        if let Some(cnonce) = self.cnonce {
            write!(f, ", cnonce={}", cnonce)?;
        }
        if let Some(nc) = self.nc {
            write!(f, ", nc={}", nc)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"nextnonce=\"47364c23432d2e131a5fb210812c\"\r\n";
        let mut reader = Reader::new(src);
        let auth_info = AuthenticationInfo::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
        assert_eq!(auth_info.nextnonce, Some("47364c23432d2e131a5fb210812c"));

        let src = b"nextnonce=\"5ccc069c403ebaf9f0171e9517f40e41\", \
        cnonce=\"0a4f113b\", nc=00000001, \
        qop=\"auth\", \
        rspauth=\"6629fae49393a05397450978507c4ef1\"\r\n";
        let mut reader = Reader::new(src);
        let auth_info = AuthenticationInfo::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");
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
