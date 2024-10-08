use core::str;

use crate::{
    macros::{parse_param, read_until_byte, read_while, sip_parse_error, space},
    parser::Result,
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

use super::SipHeaderParser;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct MimeType<'a> {
    pub mtype: &'a str,
    pub subtype: &'a str,
}
#[derive(PartialEq, Debug)]
pub struct MediaType<'a> {
    pub mimetype: MimeType<'a>,
    pub param: Option<Params<'a>>,
}
#[derive(PartialEq, Debug)]
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
                let mtype = read_until_byte!(scanner, b'/');
                scanner.next();
                let subtype =
                    read_while!(scanner, |c: u8| c != b',' && !is_newline(c) && c != b';');

                let param = parse_param!(scanner, |param| Some(param));
                let media_type = MediaType {
                    mimetype: MimeType {
                        mtype: str::from_utf8(mtype)?,
                        subtype: str::from_utf8(subtype)?,
                    },
                    param,
                };
                mtypes.push(media_type);
                scanner.read_if_eq(b',')?;
                space!(scanner);
            }
        }

        Ok(Accept {
            media_types: mtypes,
        })
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"application/sdp;level=1, application/x-private, text/html\r\n";
        let mut sc = Scanner::new(src);
        let mut params = Params::new();
        params.set("level", Some("1"));

        assert_eq!(
            Accept::parse(&mut sc),
            Ok(Accept {
                media_types: vec![
                    MediaType {
                        mimetype: MimeType {
                            mtype: "application",
                            subtype: "sdp"
                        },
                        param: Some(params)
                    },
                    MediaType {
                        mimetype: MimeType {
                            mtype: "application",
                            subtype: "x-private"
                        },
                        param: None
                    },
                    MediaType {
                        mimetype: MimeType {
                            mtype: "text",
                            subtype: "html"
                        },
                        param: None
                    }
                ]
            })
        );
    }
}
