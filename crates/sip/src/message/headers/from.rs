use crate::{
    bytes::Bytes,
    headers::TAG_PARAM,
    macros::parse_header_param,
    parser::Result,
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;

use std::str;
#[derive(Debug, PartialEq, Eq)]
pub struct From<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for From<'a> {
    const NAME: &'static [u8] = b"From";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"f");

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let uri = SipUri::parse(bytes)?;
        let mut tag = None;
        let params = parse_header_param!(bytes, TAG_PARAM = tag);

        Ok(From { tag, uri, params })
    }
}

#[cfg(test)]
mod tests {
    use crate::uri::{HostPort, Scheme, UserInfo};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"\"A. G. Bell\" <sip:agb@bell-telephone.com> ;tag=a48s\r\n";
        let mut bytes = Bytes::new(src);
        let from = From::parse(&mut bytes).unwrap();
        assert_matches!(from, From { uri: SipUri::NameAddr(addr), tag, .. } => {
            assert_eq!(addr.display, Some("A. G. Bell"));
            assert_eq!(addr.uri.user, Some(UserInfo { user: "agb", password: None }));
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "bell-telephone.com", port: None });
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("a48s"));
        });

        let src = b"sip:+12125551212@server.phone2net.com;tag=887s\r\n";
        let mut bytes = Bytes::new(src);
        let from = From::parse(&mut bytes).unwrap();
        assert_matches!(from, From { uri: SipUri::Uri(uri), tag, .. } => {
            assert_eq!(uri.user, Some(UserInfo { user: "+12125551212", password: None }));
            assert_eq!(uri.host, HostPort::DomainName { host: "server.phone2net.com", port: None });
            assert_eq!(uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("887s"));
        });

        let src = b"Anonymous <sip:c8oqz84zk7z@privacy.org>;tag=hyh8\r\n";
        let mut bytes = Bytes::new(src);
        let from = From::parse(&mut bytes).unwrap();
        assert_matches!(from, From { uri: SipUri::NameAddr(addr), tag, .. } => {
            assert_eq!(addr.display, Some("Anonymous"));
            assert_eq!(addr.uri.user, Some(UserInfo { user: "c8oqz84zk7z", password: None }));
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "privacy.org", port: None });
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(tag, Some("hyh8"));
        });
    }
}
