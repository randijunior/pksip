use std::str;

use reader::Reader;

use crate::parser::Result;

use super::{is_pass, is_user};
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) user: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    fn exist_user(reader: &mut Reader<'a>) -> bool {
        reader.peek_while(|b| !matches!(b, &b'@' | &b'\n' | &b'>' | &b' '))
            == Some(&b'@')
    }

    fn read_user(reader: &mut Reader<'a>) -> &'a str {
        unsafe { reader.read_as_str(is_user) }
    }

    fn read_pass(reader: &mut Reader<'a>) -> &'a str {
        unsafe { reader.read_as_str(is_pass) }
    }

    pub(crate) fn parse(reader: &mut Reader<'a>) -> Result<Option<Self>> {
        if !Self::exist_user(reader) {
            return Ok(None);
        }
        let user = Self::read_user(reader);
        let mut password = None;
        if reader.next() == Some(&b':') {
            let b = Self::read_pass(reader);
            reader.next();
            password = Some(b);
        }

        Ok(Some(UserInfo { user, password }))
    }
}
