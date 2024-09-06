use core::str;

use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, sip_parse_error},
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
                    let param = parse_param!(reader, Accept, |param| Some(param));
                    let media_type = MediaType {
                        mimetype: MimeType {
                            mtype: mtype?,
                            subtype: subtype?,
                        },
                        param,
                    };
                    mtypes.push(media_type);
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
