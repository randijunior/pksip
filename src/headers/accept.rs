use core::str;

use crate::{
    scanner::Scanner,
    macros::{parse_param, read_while, sip_parse_error},
    parser::Result,
    uri::Params,
    util::is_newline,
};

use super::SipHeaderParser;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}

pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}

pub struct Accept<'a> {
    media_types: Vec<MediaType<'a>>,
}

impl<'a> SipHeaderParser<'a> for Accept<'a> {
    const NAME: &'static [u8] = b"Accept";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Accept<'a>> {
        let mut mtypes: Vec<MediaType<'a>> = Vec::new();
        loop {
            if scanner.is_eof() {
                break;
            }
            if let Some(&c) = scanner.peek() {
                if is_newline(c) {
                    break;
                }
                let mtype = read_while!(scanner, |c: u8| c != b',' && !is_newline(c));
                let mut iter = mtype.split(|&b| b == b'/').map(str::from_utf8);

                if let (Some(mtype), Some(subtype)) = (iter.next(), iter.next()) {
                    let param = parse_param!(scanner, |param| Some(param));
                    let media_type = MediaType {
                        mimetype: MimeType {
                            mtype: mtype?,
                            subtype: subtype?,
                        },
                        param,
                    };
                    mtypes.push(media_type);
                } else {
                    return sip_parse_error!("Invalid Accept scanner!");
                }
            }
        }

        Ok(Accept {
            media_types: mtypes,
        })
    }
}
