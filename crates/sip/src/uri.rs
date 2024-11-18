//! Uri that appear in sip request and responses
//!

use std::str::{self};

pub(crate) use host::HostPort;
pub(crate) use crate::params::Params;
pub(crate) use scheme::Scheme;
pub(crate) use user::UserInfo;

use crate::{
    bytes::Bytes,
    headers::{parse_param_sip, Param},
    macros::{b_map, parse_param, space, until_byte},
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

fn parse_uri_param<'a>(bytes: &mut Bytes<'a>) -> Result<Param<'a>> {
    unsafe { parse_param_sip(bytes, is_param) }
}


#[derive(Debug)]
pub enum SipUri<'a> {
    Uri(Uri<'a>),
    NameAddr(NameAddr<'a>),
}

impl<'a> SipUri<'a> {
    pub(crate) fn parse(bytes: &mut Bytes<'a>) -> Result<SipUri<'a>> {
        space!(bytes);

        if matches!(bytes.peek_n(3), Some(SIP) | Some(SIPS)) {
            let uri = Uri::parse(bytes, false)?;

            return Ok(SipUri::Uri(uri));
        }

        let addr = NameAddr::parse(bytes)?;
        Ok(SipUri::NameAddr(addr))
    }
}

#[derive(Debug)]
pub struct Uri<'a> {
    pub(crate) scheme: Scheme,
    pub(crate) user: Option<UserInfo<'a>>,
    pub(crate) host: HostPort<'a>,
    pub(crate) params: Option<UriParams<'a>>,
    pub(crate) other_params: Option<Params<'a>>,
    pub(crate) header_params: Option<Params<'a>>,
}

impl<'a> Uri<'a> {
    pub(crate) fn parse(
        bytes: &mut Bytes<'a>,
        parse_params: bool,
    ) -> Result<Self> {
        let scheme = Scheme::parse(bytes)?;
        // take ':'
        bytes.next();

        let user = UserInfo::parse(bytes)?;
        let host = HostPort::parse(bytes)?;

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

        let mut uri_params = UriParams::default();
        let others = parse_param!(
            bytes,
            parse_uri_param,
            USER_PARAM = uri_params.user,
            METHOD_PARAM = uri_params.method,
            TRANSPORT_PARAM = uri_params.transport,
            TTL_PARAM = uri_params.ttl,
            LR_PARAM = uri_params.lr,
            MADDR_PARAM = uri_params.maddr
        );

        let mut header_params = None;
        if bytes.peek() == Some(&b'?') {
            let mut params = Params::new();
            loop {
                // take '?' or '&'
                bytes.next();
                let (name, value) = unsafe { parse_param_sip(bytes, is_hdr)? };
                params.set(name, value.unwrap_or(""));
                if bytes.peek() != Some(&b'&') {
                    break;
                }
            }

            header_params = Some(params)
        }

        Ok(Uri {
            scheme,
            user,
            host,
            params: Some(uri_params),
            other_params: others,
            header_params,
        })
    }
}

#[derive(Default, Debug)]
pub struct UriParams<'a> {
    pub(crate) user: Option<&'a str>,
    pub(crate) method: Option<&'a str>,
    pub(crate) transport: Option<&'a str>,
    pub(crate) ttl: Option<&'a str>,
    pub(crate) lr: Option<&'a str>,
    pub(crate) maddr: Option<&'a str>,
}

// SIP name-addr, which typically appear in From, To, and Contact header.
// display optional display part
// Struct Uri uri
#[derive(Debug)]
pub struct NameAddr<'a> {
    pub(crate) display: Option<&'a str>,
    pub(crate) uri: Uri<'a>,
}

impl<'a> NameAddr<'a> {
    pub fn parse(bytes: &mut Bytes<'a>) -> Result<NameAddr<'a>> {
        space!(bytes);
        let display = match bytes.lookahead()? {
            &b'"' => {
                bytes.next();
                let display = until_byte!(bytes, &b'"');
                bytes.must_read(b'"')?;

                Some(str::from_utf8(display)?)
            }
            &b'<' => None,
            _ => {
                let d = Token::parse(bytes);
                space!(bytes);

                Some(d)
            }
        };
        space!(bytes);
        // must be an '<'
        bytes.must_read(b'<')?;
        let uri = Uri::parse(bytes, true)?;
        // must be an '>'
        bytes.must_read(b'>')?;

        Ok(NameAddr { display, uri })
    }
}


pub struct GenericUri<'a> {
    pub(crate) scheme: &'a str,
    pub(crate) content: &'a str,
}
