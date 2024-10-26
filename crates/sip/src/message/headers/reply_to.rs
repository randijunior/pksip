use crate::{
    bytes::Bytes,
    macros::parse_header_param,
    parser::Result,
    uri::{Params, SipUri},
};

use crate::headers::SipHeaderParser;


pub struct ReplyTo<'a> {
    uri: SipUri<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ReplyTo<'a> {
    const NAME: &'static [u8] = b"Reply-To";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let uri = SipUri::parse(bytes)?;
        let param = parse_header_param!(bytes);

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
        let mut bytes = Bytes::new(src);
        let reply_to = ReplyTo::parse(&mut bytes);
        let reply_to = reply_to.unwrap();

        match reply_to {
            ReplyTo { uri: SipUri::NameAddr(addr), .. } => {
                assert_eq!(addr.uri.scheme, Scheme::Sip);
                assert_eq!(addr.uri.user.unwrap().user, "bob");
                assert_eq!(addr.uri.host, HostPort::DomainName { host: "biloxi.com", port: None });
            },
            _ => unreachable!()
        }
    }
}
