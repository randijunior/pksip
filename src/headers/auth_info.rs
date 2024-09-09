use crate::{
    byte_reader::ByteReader,
    macros::{read_until_byte, read_while, space},
    parser::{is_token, Result},
    uri::Params,
};

use super::SipHeaderParser;


use std::str;

pub struct AuthenticationInfo<'a> {
    auth_info: Params<'a>,
}

impl<'a> AuthenticationInfo<'a> {
    fn parse(reader: &mut ByteReader<'a>, params: &mut Params<'a>) -> Result<()> {
        let name = read_while!(reader,is_token);
        let name = unsafe { str::from_utf8_unchecked(name) };
        let value = if reader.peek() == Some(&b'=') {
            reader.next();
            match reader.peek() {
                Some(b'"') => {
                    reader.next();
                    let value = read_until_byte!(reader, b'"');
                    Some(str::from_utf8(value)?)
                },
                Some(_) => {
                    let value = read_while!(reader, is_token);
                    Some(unsafe { str::from_utf8_unchecked(value) })
                },
                None => None,
            }
        } else {
            None
        };

        params.set(name, value);
        
        Ok(())
    }
}

impl<'a> SipHeaderParser<'a> for AuthenticationInfo<'a> {
    const NAME: &'a [u8] = b"Authentication-Info";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        space!(reader);
        let mut params = Params::default();

        Self::parse(reader, &mut params)?;
        while let Some(b',') = reader.peek() {
            reader.next();
            Self::parse(reader, &mut params)?;
            space!(reader);
        }

        Ok(AuthenticationInfo { auth_info: params })
    }
}
