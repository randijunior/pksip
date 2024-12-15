use reader::Reader;

use crate::{
    macros::parse_header_param,
    msg::{Params, SipUri},
    parser::{self, Result},
};

use crate::headers::SipHeader;

/// The `Reply-To` SIP header.
///
/// Contains a logical return URI that may be different from the From header field
#[derive(Debug, PartialEq, Eq)]
pub struct ReplyTo<'a> {
    uri: SipUri<'a>,
    param: Option<Params<'a>>,
}

impl<'a> SipHeader<'a> for ReplyTo<'a> {
    const NAME: &'static str = "Reply-To";

    fn parse(reader: &mut Reader<'a>) -> Result<Self> {
        let uri = parser::parse_sip_uri(reader, false)?;
        let param = parse_header_param!(reader);

        Ok(ReplyTo { uri, param })
    }
}

#[cfg(test)]
mod tests {
    use crate::msg::{Host, HostPort, Scheme};

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Bob <sip:bob@biloxi.com>\r\n";
        let mut reader = Reader::new(src);
        let reply_to = ReplyTo::parse(&mut reader);
        let reply_to = reply_to.unwrap();

        assert_matches!(reply_to, ReplyTo {
            uri: SipUri::NameAddr(addr),
            ..
        } => {
            assert_eq!(addr.uri.scheme, Scheme::Sip);
            assert_eq!(addr.uri.user.unwrap().get_user(), "bob");
            assert_eq!(
                addr.uri.host_port,
                HostPort {
                    host: Host::DomainName("biloxi.com"),
                    port: None
                }
            );
        });
    }
}
