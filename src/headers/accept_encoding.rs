use std::str;

use crate::{
    byte_reader::ByteReader,
    macros::{parse_param, peek, read_while, space},
    parser::{is_token, Param, Result, Q_PARAM},
    uri::Params,
};

use super::SipHeaderParser;

#[derive(Debug, PartialEq)]
pub struct Coding<'a> {
    content_coding: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        space!(reader);
        let coding = read_while!(reader, is_token);
        let content_coding = unsafe { str::from_utf8_unchecked(coding) };
        let mut q = None;
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
#[derive(Debug, PartialEq, Default)]
pub struct AcceptEncoding<'a>(Vec<Coding<'a>>);

impl<'a> SipHeaderParser<'a> for AcceptEncoding<'a> {
    const NAME: &'static [u8] = b"Accept-Encoding";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        space!(reader);
        if reader.is_eof() {
            return Ok(AcceptEncoding::default());
        }
        let mut codings: Vec<Coding> = Vec::new();
        let coding = Coding::parse(reader)?;
        codings.push(coding);

        while let Ok(b',') = peek!(reader) {
            reader.next();
            let coding = Coding::parse(reader)?;
            codings.push(coding);
        }

        Ok(AcceptEncoding(codings))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        let src = b"compress, gzip\r\n";
        let mut reader = ByteReader::new(src);
        assert_eq!(
            AcceptEncoding::parse(&mut reader),
            Ok(AcceptEncoding(vec![
                Coding {
                    content_coding: "compress",
                    q: None,
                    param: None
                },
                Coding {
                    content_coding: "gzip",
                    q: None,
                    param: None
                },
            ]))
        );

        let mut reader = ByteReader::new(b"*\r\n");
        assert_eq!(
            AcceptEncoding::parse(&mut reader),
            Ok(AcceptEncoding(vec![Coding {
                content_coding: "*",
                q: None,
                param: None
            }]))
        );

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut reader = ByteReader::new(src);
        assert_eq!(
            AcceptEncoding::parse(&mut reader),
            Ok(AcceptEncoding(vec![
                Coding {
                    content_coding: "gzip",
                    q: Some(1.0),
                    param: None
                },
                Coding {
                    content_coding: "identity",
                    q: Some(0.5),
                    param: None
                },
                Coding {
                    content_coding: "*",
                    q: Some(0.0),
                    param: None
                }
            ]))
        );
    }
}
