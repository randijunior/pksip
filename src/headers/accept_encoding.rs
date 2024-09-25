use std::str;

use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, read_while, space},
    parser::{is_token, Param, Result, Q_PARAM},
    uri::Params,
};

use super::SipHeaderParser;

pub struct Coding<'a> {
    content_coding: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let coding = read_while!(reader, is_token);
        let content_coding = unsafe { str::from_utf8_unchecked(coding) };
        let mut q: Option<f32> = None;
        let param = parse_param!(reader, |param: Param<'a>| {
            let (name, value) = param;
            if name == Q_PARAM {
                q = AcceptEncoding::parse_q_value(value);
                return None;
            }
            Some(param)
        });
        Ok(Coding {
            content_coding,
            q,
            param,
        })
    }
}

// Accept-Encoding  =  "Accept-Encoding" HCOLON
//                     [ encoding *(COMMA encoding) ]
// encoding         =  codings *(SEMI accept-param)
// codings          =  content-coding / "*"
// content-coding   =  token
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> SipHeaderParser<'a> for AcceptEncoding<'a> {
    const NAME: &'static [u8] = b"Accept-Encoding";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut codings: Vec<Coding> = Vec::new();
        let coding = Coding::parse(reader)?;
        codings.push(coding);

        while let Some(b',') = reader.peek() {
            reader.next();
            let coding = Coding::parse(reader)?;
            codings.push(coding);
            space!(reader);
        }

        Ok(AcceptEncoding(codings))
    }
}
