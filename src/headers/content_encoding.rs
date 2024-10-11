use core::str;

use crate::{
    scanner::Scanner,
    macros::{read_while, space},
    parser::{is_token, Result},
};

use super::SipHeaderParser;
#[derive(Debug)]
pub struct ContentEncoding<'a>(Vec<&'a str>);

impl<'a> ContentEncoding<'a> {
    pub fn get(&self, index: usize) -> Option<&'a str> {
        self.0.get(index).copied()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> SipHeaderParser<'a> for ContentEncoding<'a> {
    const NAME: &'static [u8] = b"Content-Encoding";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"e");

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut codings: Vec<&'a str> = Vec::new();
        let coding = read_while!(scanner, is_token);
        let content_coding = unsafe { str::from_utf8_unchecked(coding) };
        codings.push(content_coding);

        space!(scanner);
        while let Some(b',') = scanner.peek() {
            scanner.next();
            space!(scanner);
            let coding = read_while!(scanner, is_token);
            let content_coding = unsafe { str::from_utf8_unchecked(coding) };
            codings.push(content_coding);
        }

        Ok(ContentEncoding(codings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"gzip\r\n";
        let mut scanner = Scanner::new(src);
        let c_enconding = ContentEncoding::parse(&mut scanner).unwrap();

        assert!(c_enconding.len() == 1);
        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_enconding.get(0), Some("gzip"));

        let src = b"gzip, deflate\r\n";
        let mut scanner = Scanner::new(src);
        let c_enconding = ContentEncoding::parse(&mut scanner).unwrap();
        
        assert!(c_enconding.len() == 2);
        assert_eq!(scanner.as_ref(), b"\r\n");
        assert_eq!(c_enconding.get(0), Some("gzip"));
        assert_eq!(c_enconding.get(1), Some("deflate"));
    }
}
