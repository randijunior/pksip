//! Uri that appear in sip request and responses
//!

use std::str::{self};

pub(crate) use crate::params::Params;
pub(crate) use host::HostPort;
use scanner::{space, until_byte, Scanner};
pub(crate) use scheme::Scheme;
pub(crate) use user::UserInfo;

use crate::{
    headers::{parse_param_sip, Param},
    macros::{b_map, parse_param},
    parser::{
        Result, ALPHA_NUM, ESCAPED, GENERIC_URI, HOST, PASS, UNRESERVED,
        USER_UNRESERVED,
    },
    token::Token,
};

mod host;
mod scheme;
mod user;

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

b_map!(URI_SPEC_MAP => ALPHA_NUM, GENERIC_URI);

const USER_PARAM: &str = "user";
const METHOD_PARAM: &str = "method";
const TRANSPORT_PARAM: &str = "transport";
const TTL_PARAM: &str = "ttl";
const LR_PARAM: &str = "lr";
const MADDR_PARAM: &str = "maddr";

pub(crate) const SIP: &[u8] = b"sip";
pub(crate) const SIPS: &[u8] = b"sips";

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

#[inline(always)]
pub(crate) fn is_uri(b: &u8) -> bool {
    URI_SPEC_MAP[*b as usize]
}

fn parse_uri_param<'a>(scanner: &mut Scanner<'a>) -> Result<Param<'a>> {
    let (name, value) = unsafe { parse_param_sip(scanner, is_param)? };

    Ok((name, Some(value.unwrap_or(""))))
}

#[derive(Debug, PartialEq, Eq)]
pub enum SipUri<'a> {
    Uri(Uri<'a>),
    NameAddr(NameAddr<'a>),
}

impl<'a> SipUri<'a> {
    pub(crate) fn parse(scanner: &mut Scanner<'a>) -> Result<SipUri<'a>> {
        space!(scanner);

        if matches!(scanner.peek_n(3), Some(SIP) | Some(SIPS)) {
            let uri = Uri::parse(scanner, false)?;

            return Ok(SipUri::Uri(uri));
        }

        let addr = NameAddr::parse(scanner)?;
        Ok(SipUri::NameAddr(addr))
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Uri<'a> {
    pub(crate) scheme: Scheme,
    pub(crate) user: Option<UserInfo<'a>>,
    pub(crate) host: HostPort<'a>,
    pub(crate) user_param: Option<&'a str>,
    pub(crate) method_param: Option<&'a str>,
    pub(crate) transport_param: Option<&'a str>,
    pub(crate) ttl_param: Option<&'a str>,
    pub(crate) lr_param: Option<&'a str>,
    pub(crate) maddr_param: Option<&'a str>,
    pub(crate) params: Option<Params<'a>>,
    pub(crate) hdr_params: Option<Params<'a>>,
}

impl<'a> Uri<'a> {
    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
        parse_params: bool,
    ) -> Result<Self> {
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
                ..Default::default()
            });
        }

        let mut user_param = None;
        let mut method_param = None;
        let mut transport_param = None;
        let mut ttl_param = None;
        let mut lr_param = None;
        let mut maddr_param = None;

        let params = parse_param!(
            scanner,
            parse_uri_param,
            USER_PARAM = user_param,
            METHOD_PARAM = method_param,
            TRANSPORT_PARAM = transport_param,
            TTL_PARAM = ttl_param,
            LR_PARAM = lr_param,
            MADDR_PARAM = maddr_param
        );

        let mut hdr_params = None;
        if scanner.peek() == Some(&b'?') {
            let mut params = Params::new();
            loop {
                // take '?' or '&'
                scanner.next();
                let (name, value) =
                    unsafe { parse_param_sip(scanner, is_hdr)? };
                params.set(name, value.unwrap_or(""));
                if scanner.peek() != Some(&b'&') {
                    break;
                }
            }

            hdr_params = Some(params)
        }

        Ok(Uri {
            scheme,
            user,
            host,
            user_param,
            method_param,
            transport_param,
            ttl_param,
            lr_param,
            maddr_param,
            params,
            hdr_params,
        })
    }
}

// SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
#[derive(Debug, PartialEq, Eq)]
pub struct NameAddr<'a> {
    pub(crate) display: Option<&'a str>,
    pub(crate) uri: Uri<'a>,
}

impl<'a> NameAddr<'a> {
    pub fn parse(scanner: &mut Scanner<'a>) -> Result<NameAddr<'a>> {
        space!(scanner);
        let display = match scanner.lookahead()? {
            &b'"' => {
                scanner.next();
                let display = until_byte!(scanner, &b'"');
                scanner.must_read(b'"')?;

                Some(str::from_utf8(display)?)
            }
            &b'<' => None,
            _ => {
                let d = Token::parse(scanner);
                space!(scanner);

                Some(d)
            }
        };
        space!(scanner);
        // must be an '<'
        scanner.must_read(b'<')?;
        let uri = Uri::parse(scanner, true)?;
        // must be an '>'
        scanner.must_read(b'>')?;

        Ok(NameAddr { display, uri })
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct GenericUri<'a> {
    pub(crate) scheme: &'a str,
    pub(crate) content: &'a str,
}
