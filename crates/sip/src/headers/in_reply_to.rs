use std::str;

use crate::{
    headers::{call_id::CallId, SipHeader},
    macros::{parse_header_list, read_while},
    parser::Result,
    scanner::Scanner,
    util::not_comma_or_newline,
};

/// The `In-Reply-To` SIP header.
///
/// Enumerates the `Call-IDs` that this call references or returns.
pub struct InReplyTo<'a>(Vec<CallId<'a>>);

impl<'a> SipHeader<'a> for InReplyTo<'a> {
    const NAME: &'static str = "In-Reply-To";

    fn parse(scanner: &mut Scanner<'a>) -> Result<InReplyTo<'a>> {
        let ids = parse_header_list!(scanner => {
            let id = read_while!(scanner, not_comma_or_newline);
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
        let mut scanner = Scanner::new(src);
        let in_reply_to = InReplyTo::parse(&mut scanner).unwrap();
        assert_eq!(scanner.as_ref(), b"\r\n");

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
