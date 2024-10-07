use std::str;

use crate::{
    macros::{parse_param, read_while, space},
    parser::{is_token, Param, Result, Q_PARAM},
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

use super::SipHeaderParser;

#[derive(Debug, PartialEq)]
pub struct Coding<'a> {
    content_coding: &'a str,
    q: Option<f32>,
    param: Option<Params<'a>>,
}

impl<'a> Coding<'a> {
    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        space!(scanner);
        let coding = read_while!(scanner, is_token);
        let content_coding = unsafe { str::from_utf8_unchecked(coding) };
        let mut q = None;
        let param = parse_param!(scanner, |param: Param<'a>| {
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

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        space!(scanner);

        if scanner.peek().is_some_and(|&b| is_newline(b)) {
            return Ok(AcceptEncoding::default());
        }
        let mut codings: Vec<Coding> = Vec::new();
        let coding = Coding::parse(scanner)?;
        codings.push(coding);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            let coding = Coding::parse(scanner)?;
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
        let mut scanner = Scanner::new(src);
        assert_eq!(
            AcceptEncoding::parse(&mut scanner),
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

        let mut scanner = Scanner::new(b"*\r\n");
        assert_eq!(
            AcceptEncoding::parse(&mut scanner),
            Ok(AcceptEncoding(vec![Coding {
                content_coding: "*",
                q: None,
                param: None
            }]))
        );

        let src = b"gzip;q=1.0, identity; q=0.5, *;q=0\r\n";
        let mut scanner = Scanner::new(src);
        assert_eq!(
            AcceptEncoding::parse(&mut scanner),
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
