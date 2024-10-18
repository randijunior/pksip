use crate::{
    macros::parse_param,
    parser::{self, Result},
    scanner::Scanner,
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq, Eq)]
pub struct ReplyTo<'a> {
    uri: SipUri<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ReplyTo<'a> {
    const NAME: &'static [u8] = b"Reply-To";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let uri = SipUri::parse(scanner)?;
        let param = parse_param!(scanner, |param| Some(param));

        Ok(ReplyTo { uri, param })
    }
}

#[cfg(test)]
mod tests {
    use crate::uri::{HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>\r\n";
        let mut scanner = Scanner::new(src);
        let reply_to = ReplyTo::parse(&mut scanner);
        let reply_to = reply_to.unwrap();

        assert_matches!(reply_to, ReplyTo { uri: SipUri::NameAddr(addr), .. } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user, Some(crate::uri::UserInfo { user: "bob", password: None }));
            assert_eq!(addr.uri.host, HostPort::DomainName { host: "biloxi.com", port: None });
        });
    }
}
