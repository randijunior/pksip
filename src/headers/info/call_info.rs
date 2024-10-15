use crate::{
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::{Param, Result},
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

use crate::headers::SipHeaderParser;

use std::str;

/*
Call-Info   =  "Call-Info" HCOLON info *(COMMA info)
info        =  LAQUOT absoluteURI RAQUOT *( SEMI info-param)
info-param  =  ( "purpose" EQUAL ( "icon" / "info"
               / "card" / token ) ) / generic-param
*/
pub struct CallInfo<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}


impl<'a> SipHeaderParser<'a> for CallInfo<'a> {
    const NAME: &'static [u8] = b"Call-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut purpose: Option<&'a str> = None;
        // must be an '<'
        let Some(&b'<') = scanner.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        let url = read_while!(scanner, |b| !matches!(b, b'>' | b';') && !is_newline(b));
        let url = str::from_utf8(url)?;
        // must be an '>'
        let Some(&b'>') = scanner.next() else {
            return sip_parse_error!("Invalid call info!");
        };
        space!(scanner);
        let params = parse_param!(scanner, |param: Param<'a>| {
            let (name, value) = param;
            if name == "purpose" {
                purpose = value;
                return None;
            }
            Some(param)
        });

        Ok(CallInfo {
            url,
            params,
            purpose,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> \
        ;purpose=icon\r\n";
        let mut scanner = Scanner::new(src);
        let info = CallInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon"));


        let src = b"<http://www.example.com/alice/> ;purpose=info\r\n";
        let mut scanner = Scanner::new(src);
        let info = CallInfo::parse(&mut scanner).unwrap();

        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info"));

    }
}
