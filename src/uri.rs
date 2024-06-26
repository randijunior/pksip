use std::net::IpAddr;

use crate::{parser::{self, Parser}, util::ParseSipError};

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

impl Parser<'_> for Scheme {
    fn parse(input: &'_ [u8]) -> Result<Scheme, nom::Err<ParseSipError<'_>>> {
        match input {
            b"sip" => Ok(Scheme::Sip),
            b"sips" => Ok(Scheme::Sips),
            _ => Err(nom::Err::Error(ParseSipError::new(100, "Can't parse sipuri scheme")))
        }
    }
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
pub struct Uri<'uri> {
    scheme: Scheme,
    user: Option<UserInfo<'uri>>,
    host: Host<'uri>,
    port: Option<u32>,
}

impl<'uri> Parser<'uri> for Uri<'uri> {
    fn parse(input: &'uri [u8]) -> Result<Uri<'_>, nom::Err<ParseSipError<'uri>>> {
        let (_, (scheme, after_schema, host, uri_params, uri_headers)) =
            parser::request_line(input).unwrap();

        let scheme = Scheme::parse(scheme);
        
        

        todo!()
    }
}
//SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
pub struct NameAddr<'a> {
    display: Option<&'a str>,
    uri: Uri<'a>,
}
