use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{read_while, sip_parse_error},
    parser::Result,
    uri::Params,
    util::is_newline,
};

use super::SipHeaderParser;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MimeType<'a> {
    mtype: &'a str,
    subtype: &'a str,
}

pub struct MediaType<'a> {
    mimetype: MimeType<'a>,
    param: Option<Params<'a>>,
}

pub struct Accept<'a> {
    media_types: Vec<MediaType<'a>>,
}

impl<'a> SipHeaderParser<'a> for Accept<'a> {
    const NAME: &'a [u8] = b"Accept";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Accept<'a>> {
        let mut mtypes: Vec<MediaType<'a>> = Vec::new();
        loop {
            if reader.is_eof() {
                break;
            }
            if let Some(&c) = reader.peek() {
                if is_newline(c) {
                    break;
                }
                let mtype = read_while!(reader, |c: u8| c != b',' && !is_newline(c));
                let mut iter = mtype.split(|&b| b == b'/').map(str::from_utf8);

                if let (Some(mtype), Some(subtype)) = (iter.next(), iter.next()) {
                    let mut params = Params::new();
                    while let Some(&b';') = reader.peek() {
                        let (name, value) = Accept::parse_param(reader)?;
                        params.set(str::from_utf8(name)?, value);
                    }
                    let params = if params.is_empty() {
                        None
                    } else {
                        Some(params)
                    };
                    mtypes.push(MediaType {
                        mimetype: MimeType {
                            mtype: mtype?,
                            subtype: subtype?,
                        },
                        param: params,
                    })
                } else {
                    return sip_parse_error!("Invalid Accept Reader!");
                }
            }
        }

        Ok(Accept {
            media_types: mtypes,
        })
    }
}
