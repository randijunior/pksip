use core::str;

use crate::{
    macros::{digits, sip_parse_error},
    parser::Result,
    scanner::Scanner,
};

use crate::headers::SipHeaderParser;

#[derive(Debug, PartialEq)]
pub struct MimeVersion(f32);

impl<'a> SipHeaderParser<'a> for MimeVersion {
    const NAME: &'static [u8] = b"MIME-Version";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let digits = digits!(scanner);
        match unsafe { str::from_utf8_unchecked(digits) }.parse() {
            Ok(expires) => Ok(MimeVersion(expires)),
            Err(_) => return sip_parse_error!("invalid MIME-Version!"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"1.0";
        let mut scanner = Scanner::new(src);
        let mime_version = MimeVersion::parse(&mut scanner).unwrap();

        assert_eq!(mime_version, MimeVersion(1.0));
    }
}
