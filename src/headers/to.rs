use crate::uri::SipUri;

pub struct To<'a> {
    pub(crate) tag: &'a str,
    pub(crate) uri: SipUri<'a>,
}
