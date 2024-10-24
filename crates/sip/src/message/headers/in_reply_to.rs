use core::str;

use crate::{
    bytes::Bytes,
    headers::{call_id::CallId, SipHeaderParser},
    macros::{read_while, space},
    parser::Result,
    util::is_newline,
};

#[derive(Debug, PartialEq, Eq, Default)]
pub struct InReplyTo<'a>(Vec<CallId<'a>>);

impl<'a> SipHeaderParser<'a> for InReplyTo<'a> {
    const NAME: &'static [u8] = b"In-Reply-To";

    fn parse(bytes: &mut Bytes<'a>) -> Result<Self> {
        let mut ids: Vec<CallId<'a>> = Vec::new();
        let id = read_while!(bytes, |b| b != &b',' && !is_newline(b));
        let id = str::from_utf8(id)?;
        ids.push(CallId::from(id));

        while let Some(b',') = bytes.peek() {
            bytes.next();
            space!(bytes);
            let id = read_while!(bytes, |b| b != &b',' && !is_newline(b));
            let id = str::from_utf8(id)?;
            ids.push(CallId::from(id));
            space!(bytes);
        }

        Ok(InReplyTo(ids))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com\r\n";
        let mut bytes = Bytes::new(src);
        let in_reply_to = InReplyTo::parse(&mut bytes).unwrap();
        assert_eq!(bytes.as_ref(), b"\r\n");

        assert_eq!(
            in_reply_to.0.get(0).unwrap().id(),
            "70710@saturn.bell-tel.com"
        );
        assert_eq!(
            in_reply_to.0.get(1).unwrap().id(),
            "17320@saturn.bell-tel.com"
        );
    }
}
