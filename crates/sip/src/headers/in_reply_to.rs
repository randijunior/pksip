use std::{fmt, str};

use itertools::Itertools;
use reader::{read_while, util::not_comma_or_newline, Reader};

use crate::{
    headers::{call_id::CallId, SipHeader},
    macros::hdr_list,
    parser::Result,
};

/// The `In-Reply-To` SIP header.
///
/// Enumerates the `Call-IDs` that this call references or returns.
#[derive(Debug, PartialEq, Eq)]
pub struct InReplyTo(Vec<CallId>);

impl SipHeader<'_> for InReplyTo {
    const NAME: &'static str = "In-Reply-To";
    /*
     * In-Reply-To  =  "In-Reply-To" HCOLON callid *(COMMA callid)
     */
    fn parse(reader: &mut Reader) -> Result<InReplyTo> {
        let ids = hdr_list!(reader => {
            let id = read_while!(reader, not_comma_or_newline);
            let id = str::from_utf8(id)?;

            CallId::from(id)
        });

        Ok(InReplyTo(ids))
    }
}

impl fmt::Display for InReplyTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let src = b"70710@saturn.bell-tel.com, 17320@saturn.bell-tel.com\r\n";
        let mut reader = Reader::new(src);
        let in_reply_to = InReplyTo::parse(&mut reader).unwrap();
        assert_eq!(reader.as_ref(), b"\r\n");

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
