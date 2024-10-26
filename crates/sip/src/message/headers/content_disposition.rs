use crate::{
    bytes::Bytes,
    macros::{parse_header_param, read_while, space},
    parser::{is_token, Result},
    uri::Params,
};

use crate::headers::SipHeaderParser;


pub struct ContentDisposition<'a> {
    disp_type: &'a str,
    params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for ContentDisposition<'a> {
    const NAME: &'static [u8] = b"Content-Disposition";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let disp_type = read_while!(bytes, is_token);
        let disp_type = unsafe { std::str::from_utf8_unchecked(disp_type) };
        space!(bytes);
        let params = parse_header_param!(bytes);

        Ok(ContentDisposition { disp_type, params })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"session\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes).unwrap();
        assert_eq!(disp.disp_type, "session");

        let src = b"session;handling=optional\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes).unwrap();
        assert_eq!(disp.disp_type, "session");
        assert_eq!(
            disp.params.unwrap().get("handling"),
            Some(&"optional")
        );

        let src = b"attachment; filename=smime.p7s;handling=required\r\n";
        let mut bytes = Bytes::new(src);
        let disp = ContentDisposition::parse(&mut bytes).unwrap();
        assert_eq!(disp.disp_type, "attachment");
        let params = disp.params.unwrap();

        assert_eq!(
            params.get("filename"),
            Some(&"smime.p7s")
        );
        assert_eq!(
            params.get("handling"),
            Some(&"required")
        );

    }
}
