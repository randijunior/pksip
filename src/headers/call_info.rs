use crate::{
    macros::{parse_param, read_while, sip_parse_error, space},
    parser::{Param, Result},
    scanner::Scanner,
    uri::Params,
    util::is_newline,
};

use super::SipHeaderParser;

use std::str;

/*
Call-Info   =  "Call-Info" HCOLON info *(COMMA info)
info        =  LAQUOT absoluteURI RAQUOT *( SEMI info-param)
info-param  =  ( "purpose" EQUAL ( "icon" / "info"
               / "card" / token ) ) / generic-param
*/
pub struct Info<'a> {
    url: &'a str,
    purpose: Option<&'a str>,
    params: Option<Params<'a>>,
}

impl<'a> Info<'a> {
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

        Ok(Info {
            url,
            params,
            purpose,
        })
    }
}

pub struct CallInfo<'a>(Vec<Info<'a>>);

impl<'a> CallInfo<'a> {
    pub fn get(&self, index: usize) -> Option<&Info<'a>> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for CallInfo<'a> {
    const NAME: &'static [u8] = b"Call-Info";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut infos = Vec::new();
        let info = Info::parse(scanner)?;
        infos.push(info);
        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);

            let info = Info::parse(scanner)?;

            infos.push(info);
        }

        Ok(CallInfo(infos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"<http://wwww.example.com/alice/photo.jpg> ;purpose=icon, <http://www.example.com/alice/> ;purpose=info\r\n";
        
        let mut scanner = Scanner::new(src);
        let call_info = CallInfo::parse(&mut scanner).unwrap();

        assert_eq!(scanner.as_ref(), b"\r\n");
        let info = call_info.get(0).unwrap();
        assert_eq!(info.url, "http://wwww.example.com/alice/photo.jpg");
        assert_eq!(info.purpose, Some("icon"));

        let info = call_info.get(1).unwrap();
        assert_eq!(info.url, "http://www.example.com/alice/");
        assert_eq!(info.purpose, Some("info"));

    }
}
