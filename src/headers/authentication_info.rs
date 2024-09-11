use crate::{
    byte_reader::ByteReader,
    macros::{read_until_byte, read_while, sip_parse_error, space},
    parser::{is_token, Result},
    uri::Params,
    util::not_comma_or_newline,
};

use super::SipHeaderParser;

use std::str;

// Authentication-Info: nextnonce="12345abcde67890fghij12345klmno678", rspauth="1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p", qop="auth-int", cnonce="1a2b3c4d", nc=00000002
pub struct AuthenticationInfo<'a> {
    nextnonce: Option<&'a str>,
    qop: Option<&'a str>,
    rspauth: Option<&'a str>,
    cnonce: Option<&'a str>,
    nc: Option<&'a str>
}


impl<'a> SipHeaderParser<'a> for AuthenticationInfo<'a> {
    const NAME: &'a [u8] = b"Authentication-Info";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        space!(reader);
        let mut nextnonce: Option<&'a str> = None;
        let mut rspauth: Option<&'a str> = None;
        let mut qop: Option<&'a str> = None;
        let mut cnonce: Option<&'a str> = None;
        let mut nc: Option<&'a str> = None;


        macro_rules! val {
            () => {{
                if reader.peek() == Some(&b'=') {
                    reader.next();
                    match reader.peek() {
                        Some(b'"') => {
                            reader.next();
                            let value = read_until_byte!(reader, b'"');
                            Some(str::from_utf8(value)?)
                        }
                        Some(_) => {
                            let value = read_while!(reader, is_token);
                            Some(unsafe { str::from_utf8_unchecked(value) })
                        }
                        None => None,
                    }
                } else {
                    None
                }
            }};
        }

        macro_rules! parse {
            () => {
                match read_while!(reader, not_comma_or_newline) {
                    b"nextnonce" => nextnonce = val!(),
                    b"qpop" => qop = val!(),
                    b"rspauth" => rspauth = val!(),
                    b"cnonce" => cnonce = val!(),
                    b"nc" => nc = val!(),
                    _ => sip_parse_error!("Can't parse Authentication-Info")?
                };
            };
        }

        parse!();
        while let Some(b',') = reader.peek() {
            reader.next();
            parse!();
            space!(reader);
        }

        Ok(AuthenticationInfo { nextnonce, qop, rspauth, cnonce, nc  })
    }
}
