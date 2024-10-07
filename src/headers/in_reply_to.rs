use crate::{scanner::Scanner, macros::space, parser::Result};

use super::{CallId, SipHeaderParser};

pub struct InReplyTo<'a>(Vec<CallId<'a>>);

impl<'a> SipHeaderParser<'a> for InReplyTo<'a> {
    const NAME: &'static [u8] = b"In-Reply-To";

    fn parse(scanner: &mut Scanner<'a>) -> Result<Self> {
        let mut ids: Vec<CallId<'a>> = Vec::new();
        ids.push(CallId::parse(scanner)?);

        while let Some(b',') = scanner.peek() {
            scanner.next();
            ids.push(CallId::parse(scanner)?);
            space!(scanner);
        }

        Ok(InReplyTo(ids))
    }
}
