use core::str;

use crate::{macros::read_while, parser::SipParserError, scanner::Scanner};

use super::{is_pass, is_user};

#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) user: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    fn has_user(scanner: &Scanner) -> bool {
        let mut matched = None;
        for &byte in scanner.as_ref().iter() {
            if matches!(byte, b'@' | b' ' | b'\n' | b'>') {
                matched = Some(byte);
                break;
            }
        }
        matched == Some(b'@')
    }

    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<Option<Self>, SipParserError> {
        if !Self::has_user(scanner) {
            return Ok(None);
        }
        let bytes = read_while!(scanner, is_user);
        let user = str::from_utf8(bytes)?;
        let mut user = UserInfo {
            user,
            password: None,
        };

        if scanner.next() == Some(&b':') {
            let bytes = read_while!(scanner, is_pass);
            let bytes = str::from_utf8(bytes)?;
            scanner.next();
            user.password = Some(bytes);
        }

        Ok(Some(user))
    }
}