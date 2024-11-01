use crate::{
    bytes::Bytes, headers::SipHeader, message::auth::digest::Credential,
    parser::Result,
};

/// Consists of credentials containing the authentication information of the user agent for the proxy.
pub struct ProxyAuthorization<'a>(Credential<'a>);

impl<'a> SipHeader<'a> for ProxyAuthorization<'a> {
    const NAME: &'static str = "Proxy-Authorization";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let credential = Credential::parse(bytes)?;

        Ok(ProxyAuthorization(credential))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest username=\"Alice\", realm=\"atlanta.com\", \
        nonce=\"c60f3082ee1212b402a21831ae\", \
        response=\"245f23415f11432b3434341c022\"\r\n";
        let mut bytes = Bytes::new(src);
        let proxy_auth = ProxyAuthorization::parse(&mut bytes).unwrap();

        match proxy_auth.0 {
            Credential::Digest(digest) => {
                assert_eq!(digest.username, Some("Alice"));
                assert_eq!(digest.realm, Some("atlanta.com"));
                assert_eq!(digest.nonce, Some("c60f3082ee1212b402a21831ae"));
                assert_eq!(
                    digest.response,
                    Some("245f23415f11432b3434341c022")
                );
            }
            _ => unreachable!(),
        }
    }
}
