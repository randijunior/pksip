use std::str;

use crate::{
    bytes::Bytes,
    headers::{call_id::CallId, SipHeader},
    macros::{parse_header_list, read_while},
    parser::Result,
    util::not_comma_or_newline,
};

/// The `In-Reply-To` SIP header.
///
/// Enumerates the `Call-IDs` that this call references or returns.
pub struct InReplyTo<'a>(Vec<CallId<'a>>);

impl<'a> SipHeader<'a> for InReplyTo<'a> {
    const NAME: &'static str = "In-Reply-To";

    fn parse(bytes: &mut Bytes<'a>) -> Result<InReplyTo<'a>> {
        let ids = parse_header_list!(bytes => {
            let id = read_while!(bytes, not_comma_or_newline);
            let id = str::from_utf8(id)?;

            CallId::from(id)
        });

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
