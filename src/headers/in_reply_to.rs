use crate::{byte_reader::ByteReader, macros::space, parser::Result};

use super::{CallId, SipHeaderParser};

pub struct InReplyTo<'a>(Vec<CallId<'a>>);

impl<'a> SipHeaderParser<'a> for InReplyTo<'a> {
    const NAME: &'static [u8] = b"In-Reply-To";

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        let mut ids: Vec<CallId<'a>> = Vec::new();
        ids.push(CallId::parse(reader)?);

        while let Some(b',') = reader.peek() {
            reader.next();
            ids.push(CallId::parse(reader)?);
            space!(reader);
        }

        Ok(InReplyTo(ids))
    }
}
