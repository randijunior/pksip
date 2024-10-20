use crate::{
    headers::TAG_PARAM,
    macros::parse_param,
    parser::Result,
    scanner::Scanner,
    uri::Params,
};

pub mod call_id;
pub mod cseq;
pub mod from;
pub mod max_fowards;
pub mod to;

fn parse_fromto_param<'a>(
    scanner: &mut Scanner<'a>,
) -> Result<(Option<&'a str>, Option<Params<'a>>)> {
    let mut tag = None;
    let params = parse_param!(scanner, |param| {
        let (name, value) = param;
        if name == TAG_PARAM {
            tag = value;
            None
        } else {
            Some(param)
        }
    });

    Ok((tag, params))
}
