use crate::{
    macros::{parse_param, read_while, space},
    parser::{is_token, Result},
    scanner::Scanner,
    uri::Params,
};

use crate::headers::SipHeaderParser;


#[derive(Debug, PartialEq, Eq)]
pub struct ContentDisposition<'a> {
    disp_type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ContentDisposition<'a> {
    const NAME: &'static [u8] = b"Content-Disposition";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let disp_type = read_while!(scanner, is_token);
        let disp_type = unsafe { std::str::from_utf8_unchecked(disp_type) };
        space!(scanner);
        let params = parse_param!(scanner, |param| Some(param));

        Ok(ContentDisposition { disp_type, params })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_parse() {
        let src = b"session\r\n";
        let mut scanner = Scanner::new(src);
        let disp = ContentDisposition::parse(&mut scanner).unwrap();
        assert_eq!(disp.disp_type, "session");
        assert_eq!(disp.params, None);

        let src = b"session;handling=optional\r\n";
        let mut scanner = Scanner::new(src);
        let disp = ContentDisposition::parse(&mut scanner).unwrap();
        assert_eq!(disp.disp_type, "session");
        assert_eq!(
            disp.params,
            Some(Params::from(HashMap::from([(
                "handling",
                Some("optional")
            )])))
        );

        let src = b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut scanner = Scanner::new(src);
        let disp = ContentDisposition::parse(&mut scanner).unwrap();
        assert_eq!(disp.disp_type, "attachment");
        assert_eq!(
            disp.params,
            Some(Params::from(HashMap::from([
                ("filename", Some("smime.p7s")),
                ("handling", Some("required"))
            ])))
        );
    }
}
