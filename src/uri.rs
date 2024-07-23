use std::{
    collections::HashSet,
    net::IpAddr,
    str::{self},
};

/*
Request-URI: The Request-URI is a SIP or SIPS URI as described in
           Section 19.1 or a general URI (RFC 2396 [5]).  It indicates
           the user or service to which this request is being addressed.
           The Request-URI MUST NOT contain unescaped spaces or control
           characters and MUST NOT be enclosed in "<>".
*/
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) name: &'a str,
    pub(crate) password: Option<&'a str>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Host<'a> {
    DomainName(&'a str),
    IpAddr(IpAddr),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

// scheme
// user optional
// password optional
// str host required
// u32 port optional

// transport, maddr, ttl,user, method and lr
// str use_param  optional
// str method_param optional
// str transport_param  optional
// int ttl_param optional
// int lr_param optional
// str maddr_param optional

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum UriParam<'a> {
    User(&'a str),
    Method(&'a str),
    Transport(&'a str),
    TTL(&'a str), //TODO: add i32
    LR(&'a str), //TODO: add i32
    MADDR(&'a str),
    Others(Vec<GenericUriParam<'a>>)
}

pub(crate) const USER_PARAM: &[u8] = "user".as_bytes();
pub(crate) const METHOD_PARAM: &[u8] = "method".as_bytes();
pub(crate) const TANSPORT_PARAM: &[u8] = "transport".as_bytes();
pub(crate) const TTL_PARAM: &[u8] = "ttl".as_bytes();
pub(crate) const LR_PARAM: &[u8] = "lr".as_bytes();
pub(crate) const MADDR_PARAM: &[u8] = "maddr".as_bytes();

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GenericUriParam<'a> {
    pub(crate) name: &'a str,
    pub(crate) value: &'a str
}

// struct sip_param/other_param other parameters group together
// struct sip_param/header_param optional
// SIP URI: sip:user:password@host:port;uri-parameters?headers
// SIPS URI: sips:user:password@host:port;uri-parameters?headers
#[derive(Debug, PartialEq, Eq)]
pub struct Uri<'a> {
    pub(crate) scheme: Scheme,
    pub(crate) user: Option<UserInfo<'a>>,
    pub(crate) host: Host<'a>,
    pub(crate) port: Option<u16>,
    pub(crate) uri_params: HashSet<UriParam<'a>>,
    pub(crate) header_params: Vec<GenericUriParam<'a>>
}

//SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
pub struct NameAddr<'a> {
    display: Option<&'a str>,
    uri: Uri<'a>,
}
