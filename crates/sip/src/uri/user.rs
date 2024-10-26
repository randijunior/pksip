use core::str;

use crate::{bytes::Bytes, macros::read_while, parser::SipParserError};

use super::{is_pass, is_user};


pub struct UserInfo<'a> {
    pub(crate) user: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    fn has_user(bytes: &Bytes) -> bool {
        let mut matched = None;
        for &byte in bytes.as_ref().iter() {
            if matches!(byte, b'@' | b' ' | b'\n' | b'>') {
                matched = Some(byte);
                break;
            }
        }
        matched == Some(b'@')
    }

    pub(crate) fn parse(
        bytes: &mut Bytes<'a>,
    ) -> Result<Option<Self>, SipParserError> {
        if !Self::has_user(bytes) {
            return Ok(None);
        }
        let b = read_while!(bytes, is_user);
        let user = str::from_utf8(b)?;
        let mut user = UserInfo {
            user,
            password: None,
        };

        if bytes.next() == Some(&b':') {
            let b = read_while!(bytes, is_pass);
            let b = str::from_utf8(b)?;
            bytes.next();
            user.password = Some(b);
        }

        Ok(Some(user))
    }
}
