use std::{
    collections::HashMap,
    net::IpAddr,
    str::{self, FromStr},
};

pub(crate) use host::HostPort;
pub(crate) use params::{Params, UriParams};
pub(crate) use user::UserInfo;
pub(crate) use scheme::Scheme;

use crate::{
    macros::{b_map, digits, read_until_byte, read_while, sip_parse_error, space},
    parser::{
        is_token, SipParserError, ALPHA_NUM, ESCAPED, HOST, PASS, UNRESERVED, USER_UNRESERVED
    },
    scanner::Scanner, util::is_valid_port,
};

mod host;
mod user;
mod scheme;
mod params;

// A-Z a-z 0-9 -_.!~*'() &=+$,;?/%
// For reading user part on sip uri.
b_map!(USER_SPEC_MAP => ALPHA_NUM, UNRESERVED, USER_UNRESERVED, ESCAPED);

// A-Z a-z 0-9 -_.!~*'() &=+$,%
// For reading password part on sip uri.
b_map!(PASS_SPEC_MAP => ALPHA_NUM, UNRESERVED, ESCAPED, PASS);

// A-Z a-z 0-9 -_.
b_map!(HOST_SPEC_MAP => ALPHA_NUM, HOST);

// "[]/:&+$"  "-_.!~*'()" "%"
b_map!(PARAM_SPEC_MAP => b"[]/:&+$", ALPHA_NUM, UNRESERVED, ESCAPED);

// "[]/?:+$"  "-_.!~*'()" "%"
b_map!(HDR_SPEC_MAP => b"[]/?:+$", ALPHA_NUM, UNRESERVED, ESCAPED);

const USER_PARAM: &str = "user";
const METHOD_PARAM: &str = "method";
const TRANSPORT_PARAM: &str = "transport";
const TTL_PARAM: &str = "ttl";
const LR_PARAM: &str = "lr";
const MADDR_PARAM: &str = "maddr";


#[inline(always)]
fn is_user(b: &u8) -> bool {
    USER_SPEC_MAP[*b as usize]
}

#[inline(always)]
fn is_pass(b: &u8) -> bool {
    PASS_SPEC_MAP[*b as usize]
}

#[inline(always)]
fn is_param(b: &u8) -> bool {
    PARAM_SPEC_MAP[*b as usize]
}

#[inline(always)]
fn is_hdr(b: &u8) -> bool {
    HDR_SPEC_MAP[*b as usize]
}

#[inline(always)]
pub(crate) fn is_host(b: &u8) -> bool {
    HOST_SPEC_MAP[*b as usize]
}

const SCHEME_SIP: &[u8] = b"sip";
const SCHEME_SIPS: &[u8] = b"sips";

/*
Request-URI: The Request-URI is a SIP or SIPS URI as described in
           Section 19.1 or a general URI (RFC 2396 [5]).  It indicates
           the user or service to which this request is being addressed.
           The Request-URI MUST NOT contain unescaped spaces or control
           characters and MUST NOT be enclosed in "<>".
*/

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
    pub(crate) scheme: Scheme,
    pub(crate) user: Option<UserInfo<'a>>,
    pub(crate) host: HostPort<'a>,
    pub(crate) params: Option<UriParams<'a>>,
    pub(crate) other_params: Option<Params<'a>>,
    pub(crate) header_params: Option<Params<'a>>,
}

//SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
#[derive(Debug, PartialEq, Eq)]
pub struct NameAddr<'a> {
    pub(crate) display: Option<&'a str>,
    pub(crate) uri: Uri<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SipUri<'a> {
    Uri(Uri<'a>),
    NameAddr(NameAddr<'a>),
}

impl<'a> SipUri<'a> {
    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<SipUri<'a>, SipParserError> {
        space!(scanner);
        let peeked = scanner.peek();

        match peeked {
            // Nameaddr with quoted display name
            Some(b'"') => {
                scanner.next();
                let display = read_until_byte!(scanner, &b'"');
                scanner.next();
                let display = str::from_utf8(display)?;

                space!(scanner);

                // must be an '<'
                let Some(&b'<') = scanner.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };
                let uri = Uri::parse(scanner, true)?;
                // must be an '>'
                let Some(&b'>') = scanner.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };

                Ok(SipUri::NameAddr(NameAddr {
                    display: Some(display),
                    uri,
                }))
            }
            // NameAddr without display name
            Some(&b'<') => {
                scanner.next();
                let uri = Uri::parse(scanner, true)?;
                scanner.next();

                Ok(SipUri::NameAddr(NameAddr { display: None, uri }))
            }
            // SipUri
            Some(_)
                if matches!(
                    scanner.peek_n(3),
                    Some(SCHEME_SIP) | Some(SCHEME_SIPS)
                ) =>
            {
                let uri = Uri::parse(scanner, false)?;
                Ok(SipUri::Uri(uri))
            }
            // Nameaddr with unquoted display name
            Some(_) => {
                let display = read_while!(scanner, is_token);
                let display = unsafe { str::from_utf8_unchecked(display) };

                space!(scanner);

                // must be an '<'
                let Some(&b'<') = scanner.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };
                let uri = Uri::parse(scanner, true)?;
                // must be an '>'
                let Some(&b'>') = scanner.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };

                Ok(SipUri::NameAddr(NameAddr {
                    display: Some(display),
                    uri,
                }))
            }
            None => {
                todo!()
            }
        }
    }
}

impl<'a> Uri<'a> {
    fn parse_uri_param(
        scanner: &mut Scanner<'a>,
    ) -> Result<(Option<UriParams<'a>>, Option<Params<'a>>), SipParserError> {
        if scanner.peek() == Some(&b';') {
            let mut others = Params::new();
            let mut uri_params = UriParams::default();
            while let Some(&b';') = scanner.peek() {
                scanner.next();
                let name = read_while!(scanner, is_param);
                let name = unsafe { str::from_utf8_unchecked(name) };
                let value = if scanner.peek() == Some(&b'=') {
                    scanner.next();
                    let value = read_while!(scanner, is_param);
                    Some(unsafe { str::from_utf8_unchecked(value) })
                } else {
                    None
                };
                match name {
                    USER_PARAM => uri_params.user = value,
                    METHOD_PARAM => uri_params.method = value,
                    TRANSPORT_PARAM => uri_params.transport = value,
                    TTL_PARAM => uri_params.ttl = value,
                    LR_PARAM => uri_params.lr = value,
                    MADDR_PARAM => uri_params.maddr = value,
                    _ => {
                        others.set(name, value);
                    }
                }
            }
            let params = Some(uri_params);
            let others = if others.is_empty() {
                None
            } else {
                Some(others)
            };

            Ok((params, others))
        } else {
            Ok((None, None))
        }
    }

    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
        parse_params: bool,
    ) -> Result<Self, SipParserError> {
        let scheme = Scheme::parse(scanner)?;
        // take ':'
        scanner.next();

        let user = UserInfo::parse(scanner)?;
        let host = HostPort::parse(scanner)?;

        if !parse_params {
            return Ok(Uri {
                scheme,
                user,
                host,
                params: None,
                other_params: None,
                header_params: None,
            });
        }
        let (params, other_params) = Self::parse_uri_param(scanner)?;

        let mut header_params = None;
        if scanner.peek() == Some(&b'?') {
            let mut params = Params::new();
            loop {
                // take '?' or '&'
                scanner.next();
                let name = read_while!(scanner, is_hdr);
                let name = unsafe { str::from_utf8_unchecked(name) };
                let value = if scanner.peek() == Some(&b'=') {
                    scanner.next();
                    let value = read_while!(scanner, is_hdr);
                    Some(unsafe { str::from_utf8_unchecked(value) })
                } else {
                    None
                };
                params.set(name, value);
                if scanner.peek() != Some(&b'&') {
                    break;
                }
            }

            header_params = Some(params)
        }

        Ok(Uri {
            scheme,
            user,
            host,
            params,
            other_params,
            header_params,
        })
    }
}
