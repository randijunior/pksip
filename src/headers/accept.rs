use crate::uri::Params;

#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
pub enum MimeType<'a> {
    Sdp,
    DtmfRelay,
    Other { mtype: &'a str, subtype: &'a str },
}

pub struct MediaType<'a> {
    ctype: MimeType<'a>,
    param: Params<'a>,
}

pub struct Accept<'a> {
    media_types: Vec<MediaType<'a>>
}
