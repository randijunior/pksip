use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while},
    parser::{is_token, Result},
    uri::Params,
};

use super::{
    accept::{MediaType, MimeType},
    SipHeaderParser,
};

pub struct ContentType<'a>(MediaType<'a>);

impl<'a> SipHeaderParser<'a> for ContentType<'a> {
    const NAME: &'a [u8] = b"Content-Type";
    const SHORT_NAME: Option<&'a [u8]> = Some(b"c");

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mtype = read_while!(reader, is_token);
        let mtype = unsafe { str::from_utf8_unchecked(mtype) };
        reader.next();
        let sub = read_while!(reader, is_token);
        let sub = unsafe { str::from_utf8_unchecked(sub) };
        let param = parse_param!(reader, |param| Some(param));

        Ok(ContentType(MediaType {
            mimetype: MimeType {
                mtype,
                subtype: sub,
            },
            param,
        }))
    }
}
