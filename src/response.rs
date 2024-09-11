use crate::{headers::SipHeaders, msg::StatusLine};

pub struct Response<'a> {
    req_line: StatusLine<'a>,
    headers: SipHeaders<'a>,
    body: &'a [u8],
}