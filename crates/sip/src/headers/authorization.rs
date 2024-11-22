use reader::Reader;

use crate::{auth::Credential, parser::Result};

use super::SipHeader;

/// The `Authorization` SIP header.
///
/// Contains authentication credentials of a `UA`.
#[derive(Debug, PartialEq, Eq)]
pub struct Authorization<'a>(Credential<'a>);

impl<'a> Authorization<'a> {
    pub fn credential(&self) -> &Credential<'a> {
        &self.0
    }
}

impl<'a> SipHeader<'a> for Authorization<'a> {
    const NAME: &'static str = "Authorization";

    fn parse(reader: &mut Reader<'a>) -> Result<Authorization<'a>> {
        let credential = Credential::parse(reader)?;

        Ok(Authorization(credential))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"Digest username=\"Alice\", realm=\"atlanta.com\", \
        nonce=\"84a4cc6f3082121f32b42a2187831a9e\",\
        response=\"7587245234b3434cc3412213e5f113a5432\"\r\n";
        let mut reader = Reader::new(src);
        let auth = Authorization::parse(&mut reader).unwrap();

        assert_eq!(reader.as_ref(), b"\r\n");

        assert_matches!(auth.0, Credential::Digest { username, realm, nonce, response, ..} => {
            assert_eq!(username, Some("Alice"));
            assert_eq!(realm, Some("atlanta.com"));
            assert_eq!(
                nonce,
                Some("84a4cc6f3082121f32b42a2187831a9e")
            );
            assert_eq!(
                response,
                Some("7587245234b3434cc3412213e5f113a5432")
            );
        });
    }
}
