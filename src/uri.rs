use std::net::IpAddr;

/*
Request-URI: The Request-URI is a SIP or SIPS URI as described in
           Section 19.1 or a general URI (RFC 2396 [5]).  It indicates
           the user or service to which this request is being addressed.
           The Request-URI MUST NOT contain unescaped spaces or control
           characters and MUST NOT be enclosed in "<>".
*/
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    name: &'a str,
    password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    pub fn new(name: &'a str, pass: Option<&'a str>) -> Self {
        UserInfo {
            name,
            password: pass,
        }
    }
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

// struct sip_param/other_param other parameters group together
// struct sip_param/header_param optional
// SIP URI: sip:user:password@host:port;uri-parameters?headers
// SIPS URI: sips:user:password@host:port;uri-parameters?headers
#[derive(Debug, PartialEq, Eq)]
pub struct Uri<'a> {
    scheme: Scheme,
    user: Option<UserInfo<'a>>,
    host: Host<'a>,
    port: Option<u32>,
}

impl<'a> Uri<'a> {
    pub fn new(
        scheme: Scheme,
        user: Option<UserInfo<'a>>,
        host: Host<'a>,
        port: Option<u32>,
    ) -> Self {
        Uri {
            scheme,
            user,
            host,
            port,
        }
    }
}

//SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
pub struct NameAddr<'a> {
    display: Option<&'a str>,
    uri: Uri<'a>,
}
