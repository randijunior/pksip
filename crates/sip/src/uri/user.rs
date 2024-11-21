use std::str;

use scanner::Scanner;

use crate::parser::SipParserError;

use super::{is_pass, is_user};
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) user: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<Option<Self>, SipParserError> {
        let haystack = scanner.as_ref();
        let p = memchr::memchr3(b'@', b'\n', b'>', haystack);
        if !p.is_some_and(|b| haystack[b] == b'@') {
            return Ok(None);
        }
        let user = unsafe { scanner.read_while_as_str(is_user) };
        let mut password = None;
        if scanner.next() == Some(&b':') {
            let b = unsafe { scanner.read_while_as_str(is_pass) };
            scanner.next();
            password = Some(b);
        }

        Ok(Some(UserInfo { user, password }))
    }
}
