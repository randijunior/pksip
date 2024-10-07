use core::str;

use crate::{scanner::Scanner, macros::until_newline, parser::Result};

use super::SipHeaderParser;

pub struct Organization<'a>(&'a str);

impl<'a> SipHeaderParser<'a> for Organization<'a> {
    const NAME: &'static [u8] = b"Organization";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let organization = until_newline!(scanner);
        let organization = str::from_utf8(organization)?;

        Ok(Organization(organization))
    }
}
