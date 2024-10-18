use std::{
    collections::HashMap,
    net::IpAddr,
    str::{self, FromStr},
};

use crate::{
    macros::{b_map, digits, read_until_byte, read_while, sip_parse_error, space},
    parser::{
        is_token, SipParserError, ALPHA_NUM, ESCAPED, HOST, PASS, UNRESERVED, USER_UNRESERVED
    },
    scanner::Scanner, util::is_valid_port,
};

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
#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo<'a> {
    pub(crate) user: &'a str,
    pub(crate) password: Option<&'a str>,
}

impl<'a> UserInfo<'a> {
    fn has_user(scanner: &Scanner) -> bool {
        let mut matched = None;
        for &byte in scanner.as_ref().iter() {
            if matches!(byte, b'@' | b' ' | b'\n' | b'>') {
                matched = Some(byte);
                break;
            }
        }
        matched == Some(b'@')
    }

    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<Option<Self>, SipParserError> {
        if !Self::has_user(scanner) {
            return Ok(None);
        }
        let bytes = read_while!(scanner, is_user);
        let user = str::from_utf8(bytes)?;
        let mut user = UserInfo {
            user,
            password: None,
        };

        if scanner.next() == Some(&b':') {
            let bytes = read_while!(scanner, is_pass);
            let bytes = str::from_utf8(bytes)?;
            scanner.next();
            user.password = Some(bytes);
        }

        Ok(Some(user))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HostPort<'a> {
    DomainName { host: &'a str, port: Option<u16> },
    IpAddr { host: IpAddr, port: Option<u16> },
}

impl<'a> HostPort<'a> {
    fn parse_port(scanner: &mut Scanner) -> Result<Option<u16>, SipParserError> {
        if let Ok(Some(_)) = scanner.read_if(|b| b == &b':') {
            let digits = digits!(scanner);
            let digits = unsafe { str::from_utf8_unchecked(digits) };
            match digits.parse::<u16>() {
                Ok(port) if is_valid_port(port) => Ok(Some(port)),
                Ok(_) | Err(_) => {
                    sip_parse_error!("Sip Uri Port is invalid!")
                }
            }
        } else {
            Ok(None)
        }
    }
    pub(crate) fn parse(
        scanner: &mut Scanner<'a>,
    ) -> Result<HostPort<'a>, SipParserError> {
        if let Ok(Some(_)) = scanner.read_if(|b| b == &b'[') {
            // the '[' and ']' characters are removed from the host
            let host = read_until_byte!(scanner, &b']');
            let host = str::from_utf8(host)?;
            scanner.next();
            return if let Ok(host) = host.parse() {
                scanner.next();
                Ok(HostPort::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(scanner)?,
                })
            } else {
                sip_parse_error!("scannerError parsing Ipv6 HostPort!")
            };
        }
        let host = read_while!(scanner,is_host);
        let host = unsafe { str::from_utf8_unchecked(host) };
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(HostPort::IpAddr {
                host: addr,
                port: Self::parse_port(scanner)?,
            })
        } else {
            Ok(HostPort::DomainName {
                host,
                port: Self::parse_port(scanner)?,
            })
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Scheme {
    Sip,
    Sips,
}

impl Scheme {
    pub (crate) fn parse(scanner: &mut Scanner) -> Result<Self, SipParserError> {
        match read_until_byte!(scanner, &b':') {
            SCHEME_SIP => Ok(Scheme::Sip),
            SCHEME_SIPS => Ok(Scheme::Sips),
            // Unsupported URI scheme
            unsupported => sip_parse_error!(format!(
                "Unsupported URI scheme: {}",
                String::from_utf8_lossy(unsupported)
            )),
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

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Params<'a> {
    pub(crate) inner: HashMap<&'a str, Option<&'a str>>,
}

impl<'a> From<HashMap<&'a str, Option<&'a str>>> for Params<'a> {
    fn from(value: HashMap<&'a str, Option<&'a str>>) -> Self {
        Self { inner: value }
    }
}

impl<'a> Params<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn set(
        &mut self,
        k: &'a str,
        v: Option<&'a str>,
    ) -> Option<Option<&str>> {
        self.inner.insert(k, v)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct UriParams<'a> {
    pub(crate) user: Option<&'a str>,
    pub(crate) method: Option<&'a str>,
    pub(crate) transport: Option<&'a str>,
    pub(crate) ttl: Option<&'a str>,
    pub(crate) lr: Option<&'a str>,
    pub(crate) maddr: Option<&'a str>,
}

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
