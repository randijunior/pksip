use reader::Reader;

use crate::{
    headers::TAG_PARAM,
    macros::parse_header_param,
    message::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

use core::fmt;
use std::str;

/// The `From` SIP header.
///
/// Indicates the initiator of the request.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct From<'a> {
    pub uri: SipUri<'a>,
    pub tag: Option<&'a str>,
    pub params: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for From<'a> {
    const NAME: &'static str = "From";
    const SHORT_NAME: &'static str = "f";
    /*
     * From        =  ( "From" / "f" ) HCOLON from-spec
     * from-spec   =  ( name-addr / addr-spec )
     *                *( SEMI from-param )
     * from-param  =  tag-param / generic-param
     * tag-param   =  "tag" EQUAL token
     */
    fn parse(reader: &mut Reader<'a>) -> Result<From<'a>> {
        let uri = parser::parse_sip_uri(reader, false)?;
        let mut tag = None;
        let params = parse_header_param!(reader, TAG_PARAM = tag);

        Ok(From { tag, uri, params })
    }
}

impl fmt::Display for From<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.uri {
            SipUri::Uri(uri) => write!(f, "{}", uri)?,
            SipUri::NameAddr(name_addr) => write!(f, "{}", name_addr)?,
        }
        if let Some(tag) = self.tag {
            write!(f, ";tag={}", tag)?;
        }
        if let Some(params) = &self.params {
            write!(f, "{}", params)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::message::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"\"A. G. Bell\" <sip:agb@bell-telephone.com> ;tag=a48s\r\n";
        let mut reader = Reader::new(src);
        let from = From::parse(&mut reader).unwrap();

        assert_matches!(from, From {
            uri: SipUri::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some("A. G. Bell"));
            assert_eq!(addr.uri.user.unwrap().get_user(), "agb");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("bell-telephone.com"),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("a48s"));
        });

        let src = b"sip:+12125551212@server.phone2net.com;tag=887s\r\n";
        let mut reader = Reader::new(src);
        let from = From::parse(&mut reader).unwrap();

        assert_matches!(from, From {
            uri: SipUri::Uri(uri),
            tag,
            ..
        } => {
            assert_eq!(uri.user.unwrap().get_user(), "+12125551212");
            assert_eq!(
                uri.host_port,
                HostPort {
                    host: Host::DomainName("server.phone2net.com"),
                    port: None
                }
            );
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("887s"));
        });

        let src = b"Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8\r\n";
        let mut reader = Reader::new(src);
        let from = From::parse(&mut reader).unwrap();

        assert_matches!(from, From {
            uri: SipUri::NameAddr(addr),
            tag,
            ..
        } => {
            assert_eq!(addr.display, Some("Anonymous"));
            assert_eq!(addr.uri.user.unwrap().get_user(), "c8oqz84zk7z");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("privacy.org"),
                    port: None
                }
            );
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("hyh8"));
         });
    }
}
