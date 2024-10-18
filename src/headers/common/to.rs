use crate::{
    parser::{self, Result},
    scanner::Scanner,
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;

use std::str;
#[derive(Debug, PartialEq, Eq)]
pub struct To<'a> {
    pub(crate) uri: SipUri<'a>,
    pub(crate) tag: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for To<'a> {
    const NAME: &'static [u8] = b"To";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"t");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let uri = SipUri::parse(scanner)?;
        let (tag, params) = super::parse_fromto_param(scanner)?;

        Ok(To { tag, uri, params })
    }
}

#[cfg(test)]
mod tests {
    use crate::uri::{HostPort, Scheme, UserInfo};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>;tag=a6c85cf\r\n";
        let mut scanner = Scanner::new(src);
        let to = To::parse(&mut scanner);
        let to = to.unwrap();

        assert_matches!(to, To { uri: SipUri::NameAddr(addr), tag, .. } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.display, Some("Bob"));
            assert_eq!(addr.uri.user, Some(UserInfo { user: "bob", password: None}));
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "biloxi.com", port: None });
            assert_eq!(tag, Some("a6c85cf"));
        });
    }
}
