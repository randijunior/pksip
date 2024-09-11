use crate::{headers::SipHeaders, msg::RequestLine};

pub struct Request<'a> {
    req_line: RequestLine<'a>,
    headers: SipHeaders<'a>,
    body: &'a [u8],
}