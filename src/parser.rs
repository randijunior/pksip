use crate::headers::accept::Accept;
use crate::headers::accept_encoding::AcceptEncoding;
use crate::headers::accept_language::AcceptLanguage;
use crate::headers::alert_info::AlertInfo;
use crate::headers::allow::Allow;
use crate::headers::authentication_info::AuthenticationInfo;
use crate::headers::authorization::Authorization;
use crate::headers::call_id::CallId;
use crate::headers::contact::Contact;
use crate::headers::content_disposition::ContentDisposition;
use crate::headers::content_encoding::ContentEncoding;
use crate::headers::content_length::ContentLength;
use crate::headers::content_type::ContentType;
use crate::headers::cseq::CSeq;
use crate::headers::date::Date;
use crate::headers::error_info::ErrorInfo;
use crate::headers::expires::Expires;
use crate::headers::in_reply_to::InReplyTo;
use crate::headers::max_fowards::MaxForwards;
use crate::headers::mime_version::MimeVersion;
use crate::headers::min_expires::MinExpires;
use crate::headers::organization::Organization;
use crate::headers::priority::Priority;
use crate::headers::proxy_authenticate::ProxyAuthenticate;
use crate::headers::proxy_authorization::ProxyAuthorization;
use crate::headers::proxy_require::ProxyRequire;
use crate::headers::record_route::RecordRoute;
use crate::headers::reply_to::ReplyTo;
use crate::headers::require::Require;
use crate::headers::retry_after::RetryAfter;
use crate::headers::route::Route;
use crate::headers::server::Server;
use crate::headers::subject::Subject;
use crate::headers::supported::Supported;
use crate::headers::timestamp::Timestamp;
use crate::headers::to::To;
use crate::headers::unsupported::Unsupported;
use crate::headers::user_agent::UserAgent;
use crate::headers::via::Via;
use crate::headers::warning::Warning;
use crate::headers::www_authenticate::WWWAuthenticate;
use crate::headers::{self, SipHeaderParser};

use crate::{byte_reader::ByteReader, headers::Header};

pub type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::net::IpAddr;
use std::str::FromStr;
use std::str::Utf8Error;

use crate::byte_reader::ReaderError;
use crate::headers::via::ViaParams;
use crate::headers::SipHeaders;

use crate::macros::b_map;
use crate::macros::digits;
use crate::macros::find;
use crate::macros::newline;
use crate::macros::peek_while;
use crate::macros::read_until_byte;
use crate::macros::read_while;
use crate::macros::sip_parse_error;
use crate::macros::space;
use crate::macros::until_newline;
use crate::macros::{alpha, parse_param};

use crate::msg::RequestLine;
use crate::msg::SipMethod;
use crate::msg::SipMsg;
use crate::msg::SipStatusCode;
use crate::msg::StatusLine;

use crate::uri::HostPort;
use crate::uri::Scheme;
use crate::uri::Uri;
use crate::uri::UriParams;
use crate::uri::UserInfo;
use crate::uri::{NameAddr, Params, SipUri};
use crate::util::is_space;
use crate::util::is_valid_port;
use crate::util::{is_alphabetic, is_newline};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const UNRESERVED: &[u8] = b"-_.!~*'()%";
const ESCAPED: &[u8] = b"%";
const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
const TOKEN: &[u8] = b"-.!%*_`'~+";
const PASS: &[u8] = b"&=+$,";
const HOST: &[u8] = b"_-.";
const GENERIC_URI: &[u8] = b"#?;:@&=+-_.!~*'()%$,/";

pub(crate) const USER_PARAM: &str = "user";
pub(crate) const METHOD_PARAM: &str = "method";
pub(crate) const TRANSPORT_PARAM: &str = "transport";
pub(crate) const TTL_PARAM: &str = "ttl";
pub(crate) const LR_PARAM: &str = "lr";
pub(crate) const MADDR_PARAM: &str = "maddr";
pub(crate) const BRANCH_PARAM: &str = "branch";
pub(crate) const RPORT_PARAM: &str = "rport";
pub(crate) const RECEIVED_PARAM: &str = "received";
pub(crate) const TAG_PARAM: &str = "tag";
pub(crate) const Q_PARAM: &str = "q";
pub(crate) const EXPIRES_PARAM: &str = "expires";

pub(crate) const SCHEME_SIP: &[u8] = b"sip";
pub(crate) const SCHEME_SIPS: &[u8] = b"sips";

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

b_map!(TOKEN_SPEC_MAP => ALPHA_NUM, TOKEN);

b_map!(VIA_PARAM_SPEC_MAP => b"[:]", ALPHA_NUM, TOKEN);

b_map!(GENERIC_URI_SPEC_MAP => ALPHA_NUM, GENERIC_URI);

pub(crate) type Param<'a> = (&'a str, Option<&'a str>);

pub struct SipParser<'a> {
    reader: ByteReader<'a>,
}

impl<'a> SipParser<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        SipParser {
            reader: ByteReader::new(bytes),
        }
    }

    fn parse_scheme(reader: &mut ByteReader) -> Result<Scheme> {
        match read_until_byte!(reader, b':') {
            SCHEME_SIP => Ok(Scheme::Sip),
            SCHEME_SIPS => Ok(Scheme::Sips),
            // Unsupported URI scheme
            _ => sip_parse_error!("Can't parse sip uri scheme"),
        }
    }

    fn has_user(reader: &ByteReader) -> bool {
        let mut matched = None;
        for &byte in reader.as_ref().iter() {
            if matches!(byte, b'@' | b' ' | b'\n' | b'>') {
                matched = Some(byte);
                break;
            }
        }
        matched == Some(b'@')
    }

    fn parse_user(reader: &mut ByteReader<'a>) -> Result<Option<UserInfo<'a>>> {
        if !Self::has_user(reader) {
            return Ok(None);
        }
        let bytes = read_while!(reader, is_user);
        let name = str::from_utf8(bytes)?;
        let mut user = UserInfo {
            name,
            password: None,
        };

        if reader.next() == Some(&b':') {
            let bytes = read_while!(reader, is_pass);
            let bytes = str::from_utf8(bytes)?;
            reader.next();
            user.password = Some(bytes);
        }

        Ok(Some(user))
    }

    pub(crate) fn parse_sip_version(reader: &mut ByteReader<'a>) -> Result<()> {
        let _version = find!(reader, SIPV2);

        Ok(())
    }

    pub(crate) fn parse_fromto_param(
        reader: &mut ByteReader<'a>,
    ) -> Result<(Option<&'a str>, Option<Params<'a>>)> {
        let mut tag = None;
        let params = parse_param!(reader, |param: Param<'a>| {
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

    pub(crate) fn parse_sip_uri(reader: &mut ByteReader<'a>) -> Result<SipUri<'a>> {
        space!(reader);
        let peeked = reader.peek();

        match peeked {
            // Nameaddr with quoted display name
            Some(b'"') => {
                reader.next();
                let display = read_until_byte!(reader, b'"');
                reader.next();
                let display = str::from_utf8(display)?;

                space!(reader);

                // must be an '<'
                let Some(&b'<') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };
                let uri = Self::parse_uri(reader, true)?;
                // must be an '>'
                let Some(&b'>') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };

                Ok(SipUri::NameAddr(NameAddr {
                    display: Some(display),
                    uri,
                }))
            }
            // NameAddr without display name
            Some(&b'<') => {
                reader.next();
                let uri = Self::parse_uri(reader, true)?;
                reader.next();

                Ok(SipUri::NameAddr(NameAddr { display: None, uri }))
            }
            // SipUri
            Some(_) if reader.peek_n(3) == Some(SCHEME_SIP) => {
                let uri = Self::parse_uri(reader, false)?;
                Ok(SipUri::Uri(uri))
            }
            // Nameaddr with unquoted display name
            Some(_) => {
                let display = read_while!(reader, is_token);
                let display = unsafe { str::from_utf8_unchecked(display) };

                space!(reader);

                // must be an '<'
                let Some(&b'<') = reader.next() else {
                    return sip_parse_error!("Invalid name addr!");
                };
                let uri = Self::parse_uri(reader, true)?;
                // must be an '>'
                let Some(&b'>') = reader.next() else {
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

    pub(crate) fn parse_host(reader: &mut ByteReader<'a>) -> Result<HostPort<'a>> {
        if let Ok(Some(_)) = reader.read_if(|b| b == b'[') {
            // the '[' and ']' characters are removed from the host
            let host = read_until_byte!(reader, b']');
            let host = str::from_utf8(host)?;
            reader.next();
            return if let Ok(host) = host.parse() {
                reader.next();
                Ok(HostPort::IpAddr {
                    host: IpAddr::V6(host),
                    port: Self::parse_port(reader)?,
                })
            } else {
                sip_parse_error!("ReaderError parsing Ipv6 HostPort!")
            };
        }
        let host = read_while!(reader, |b| HOST_SPEC_MAP[b as usize]);
        let host = unsafe { str::from_utf8_unchecked(host) };
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(HostPort::IpAddr {
                host: addr,
                port: Self::parse_port(reader)?,
            })
        } else {
            Ok(HostPort::DomainName {
                host,
                port: Self::parse_port(reader)?,
            })
        }
    }

    fn parse_port(reader: &mut ByteReader) -> Result<Option<u16>> {
        if let Ok(Some(_)) = reader.read_if(|b| b == b':') {
            let digits = digits!(reader);
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

    fn parse_uri_param(
        reader: &mut ByteReader<'a>,
    ) -> Result<(Option<UriParams<'a>>, Option<Params<'a>>)> {
        if reader.peek() == Some(&b';') {
            let mut others = Params::new();
            let mut uri_params = UriParams::default();
            while let Some(&b';') = reader.peek() {
                reader.next();
                let name = read_while!(reader, is_param);
                let name = unsafe { str::from_utf8_unchecked(name) };
                let value = if reader.peek() == Some(&b'=') {
                    reader.next();
                    let value = read_while!(reader, is_param);
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

    fn parse_uri(reader: &mut ByteReader<'a>, parse_params: bool) -> Result<Uri<'a>> {
        let scheme = Self::parse_scheme(reader)?;
        // take ':'
        reader.next();

        let user = Self::parse_user(reader)?;
        let host = Self::parse_host(reader)?;

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
        let (params, other_params) = Self::parse_uri_param(reader)?;

        let mut header_params = None;
        if reader.peek() == Some(&b'?') {
            let mut params = Params::new();
            loop {
                // take '?' or '&'
                reader.next();
                let name = read_while!(reader, is_hdr);
                let name = unsafe { str::from_utf8_unchecked(name) };
                let value = if reader.peek() == Some(&b'=') {
                    reader.next();
                    let value = read_while!(reader, is_hdr);
                    Some(unsafe { str::from_utf8_unchecked(value) })
                } else {
                    None
                };
                params.set(name, value);
                if reader.peek() != Some(&b'&') {
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

    pub(crate) fn parse_via_params(
        reader: &mut ByteReader<'a>,
    ) -> Result<(Option<ViaParams<'a>>, Option<Params<'a>>)> {
        if reader.peek() != Some(&b';') {
            return Ok((None, None));
        }
        let mut params = ViaParams::default();
        let mut others = Params::new();
        while let Some(&b';') = reader.peek() {
            reader.next();
            let name = read_while!(reader, is_via_param);
            let name = unsafe { str::from_utf8_unchecked(name) };
            let mut value = "";
            if let Some(&b'=') = reader.peek() {
                reader.next();
                let v = read_while!(reader, is_via_param);
                value = unsafe { str::from_utf8_unchecked(v) };
            }
            match name {
                BRANCH_PARAM => params.set_branch(value),
                TTL_PARAM => params.set_ttl(value),
                MADDR_PARAM => params.set_maddr(value),
                RECEIVED_PARAM => params.set_received(value),
                RPORT_PARAM => {
                    if !value.is_empty() {
                        match value.parse::<u16>() {
                            Ok(port) if is_valid_port(port) => params.set_rport(port),
                            Ok(_) | Err(_) => {
                                return sip_parse_error!("Via param rport is invalid!")
                            }
                        }
                    }
                }
                _ => {
                    others.set(name, Some(value));
                }
            }
        }

        let others = if others.is_empty() {
            None
        } else {
            Some(others)
        };

        Ok((Some(params), others))
    }

    fn parse_status_line(&mut self) -> Result<StatusLine<'a>> {
        let reader = &mut self.reader;
        Self::parse_sip_version(reader)?;

        space!(reader);
        let digits = digits!(reader);
        space!(reader);

        let status_code = SipStatusCode::from(digits);
        let bytes = until_newline!(reader);

        let rp = str::from_utf8(bytes)?;

        newline!(reader);
        Ok(StatusLine::new(status_code, rp))
    }

    fn parse_request_line(&mut self) -> Result<RequestLine<'a>> {
        let reader = &mut self.reader;
        let b_method = alpha!(reader);
        let method = SipMethod::from(b_method);

        space!(reader);
        let uri = Self::parse_uri(reader, true)?;
        space!(reader);

        Self::parse_sip_version(reader)?;
        newline!(reader);

        Ok(RequestLine { method, uri })
    }

    fn is_sip_request(&self) -> bool {
        const SIP: &[u8] = b"SIP";
        let reader = &self.reader;
        let tag = peek_while!(reader, is_alphabetic);
        let next = reader.src.get(tag.len() + 1);

        next.is_some_and(|next| (next == &b'/' || is_space(*next)) && tag == SIP)
    }

    fn parse_headers(&mut self, headers: &mut SipHeaders<'a>) -> Result<()> {
        let reader = &mut self.reader;
        'headers: loop {
            let name = read_while!(reader, is_token);

            if reader.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }
            space!(reader);

            match name {
                error_info if ErrorInfo::match_name(error_info) => {
                    let error_info = ErrorInfo::parse(reader)?;
                    headers.push_header(Header::ErrorInfo(error_info))
                }
                route if Route::match_name(route) => 'route: loop {
                    let route = Route::parse(reader)?;
                    headers.push_header(Header::Route(route));
                    let Some(&b',') = reader.peek() else {
                        break 'route;
                    };
                    reader.next();
                },
                via if Via::match_name(via) => 'via: loop {
                    let via = Via::parse(reader)?;
                    headers.push_header(Header::Via(via));
                    let Some(&b',') = reader.peek() else {
                        break 'via;
                    };
                    reader.next();
                },
                max_fowards if MaxForwards::match_name(max_fowards) => {
                    let max_fowards = MaxForwards::parse(reader)?;
                    headers.push_header(Header::MaxForwards(max_fowards))
                }
                from if headers::From::match_name(from) => {
                    let from = headers::From::parse(reader)?;
                    headers.push_header(Header::From(from))
                }
                to if To::match_name(to) => {
                    let to = To::parse(reader)?;
                    headers.push_header(Header::To(to))
                }
                cid if CallId::match_name(cid) => {
                    let call_id = CallId::parse(reader)?;
                    headers.push_header(Header::CallId(call_id))
                }
                cseq if CSeq::match_name(cseq) => {
                    let cseq = CSeq::parse(reader)?;
                    headers.push_header(Header::CSeq(cseq))
                }
                auth if Authorization::match_name(auth) => {
                    let auth = Authorization::parse(reader)?;
                    headers.push_header(Header::Authorization(auth))
                }
                contact if Contact::match_name(contact) => 'contact: loop {
                    let contact = Contact::parse(reader)?;
                    headers.push_header(Header::Contact(contact));
                    let Some(&b',') = reader.peek() else {
                        break 'contact;
                    };
                    reader.next();
                },
                expires if Expires::match_name(expires) => {
                    let expires = Expires::parse(reader)?;
                    headers.push_header(Header::Expires(expires));
                }
                in_reply_to if InReplyTo::match_name(in_reply_to) => {
                    let in_reply_to = InReplyTo::parse(reader)?;
                    headers.push_header(Header::InReplyTo(in_reply_to));
                }
                mime_version if MimeVersion::match_name(mime_version) => {
                    let mime_version = MimeVersion::parse(reader)?;
                    headers.push_header(Header::MimeVersion(mime_version));
                }
                min_expires if MinExpires::match_name(min_expires) => {
                    let min_expires = MinExpires::parse(reader)?;
                    headers.push_header(Header::MinExpires(min_expires));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(reader)?;
                    headers.push_header(Header::UserAgent(user_agent))
                }
                date if Date::match_name(date) => {
                    let date = Date::parse(reader)?;
                    headers.push_header(Header::Date(date))
                }
                server if Server::match_name(server) => {
                    let server = Server::parse(reader)?;
                    headers.push_header(Header::Server(server))
                }
                subject if Subject::match_name(subject) => {
                    let subject = Subject::parse(reader)?;
                    headers.push_header(Header::Subject(subject))
                }
                priority if Priority::match_name(priority) => {
                    let priority = Priority::parse(reader)?;
                    headers.push_header(Header::Priority(priority))
                }
                proxy_authenticate if ProxyAuthenticate::match_name(proxy_authenticate) => {
                    let proxy_authenticate = ProxyAuthenticate::parse(reader)?;
                    headers.push_header(Header::ProxyAuthenticate(proxy_authenticate))
                }
                proxy_authorization if ProxyAuthorization::match_name(proxy_authorization) => {
                    let proxy_authorization = ProxyAuthorization::parse(reader)?;
                    headers.push_header(Header::ProxyAuthorization(proxy_authorization))
                }
                proxy_require if ProxyRequire::match_name(proxy_require) => {
                    let proxy_require = ProxyRequire::parse(reader)?;
                    headers.push_header(Header::ProxyRequire(proxy_require))
                }
                reply_to if ReplyTo::match_name(reply_to) => {
                    let reply_to = ReplyTo::parse(reader)?;
                    headers.push_header(Header::ReplyTo(reply_to))
                }
                content_length if ContentLength::match_name(content_length) => {
                    let content_length = ContentLength::parse(reader)?;
                    headers.push_header(Header::ContentLength(content_length))
                }
                content_encoding if ContentEncoding::match_name(content_encoding) => {
                    let content_encoding = ContentEncoding::parse(reader)?;
                    headers.push_header(Header::ContentEncoding(content_encoding))
                }
                content_type if ContentType::match_name(content_type) => {
                    let content_type = ContentType::parse(reader)?;
                    headers.push_header(Header::ContentType(content_type))
                }
                content_disposition if ContentDisposition::match_name(content_disposition) => {
                    let content_disposition = ContentDisposition::parse(reader)?;
                    headers.push_header(Header::ContentDisposition(content_disposition))
                }
                record_route if RecordRoute::match_name(record_route) => 'rr: loop {
                    let record_route = RecordRoute::parse(reader)?;
                    headers.push_header(Header::RecordRoute(record_route));
                    let Some(&b',') = reader.peek() else {
                        break 'rr;
                    };
                    reader.next();
                },
                require if Require::match_name(require) => {
                    let require = Require::parse(reader)?;
                    headers.push_header(Header::Require(require))
                }
                retry_after if RetryAfter::match_name(retry_after) => {
                    let retry_after = RetryAfter::parse(reader)?;
                    headers.push_header(Header::RetryAfter(retry_after))
                }
                organization if Organization::match_name(organization) => {
                    let organization = Organization::parse(reader)?;
                    headers.push_header(Header::Organization(organization))
                }
                accept_encoding if AcceptEncoding::match_name(accept_encoding) => {
                    let accept_encoding = AcceptEncoding::parse(reader)?;
                    headers.push_header(Header::AcceptEncoding(accept_encoding));
                }
                accept if Accept::match_name(accept) => {
                    let accept = Accept::parse(reader)?;
                    headers.push_header(Header::Accept(accept));
                }
                accept_language if AcceptLanguage::match_name(accept_language) => {
                    let accept_language = AcceptLanguage::parse(reader)?;
                    headers.push_header(Header::AcceptLanguage(accept_language));
                }
                alert_info if AlertInfo::match_name(alert_info) => {
                    let alert_info = AlertInfo::parse(reader)?;
                    headers.push_header(Header::AlertInfo(alert_info));
                }
                allow if Allow::match_name(allow) => {
                    let allow = Allow::parse(reader)?;
                    headers.push_header(Header::Allow(allow));
                }
                auth_info if AuthenticationInfo::match_name(auth_info) => {
                    let auth_info = AuthenticationInfo::parse(reader)?;
                    headers.push_header(Header::AuthenticationInfo(auth_info));
                }
                supported if Supported::match_name(supported) => {
                    let supported = Supported::parse(reader)?;
                    headers.push_header(Header::Supported(supported));
                }
                timestamp if Timestamp::match_name(timestamp) => {
                    let timestamp = Timestamp::parse(reader)?;
                    headers.push_header(Header::Timestamp(timestamp));
                }
                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(reader)?;
                    headers.push_header(Header::UserAgent(user_agent));
                }
                unsupported if Unsupported::match_name(unsupported) => {
                    let unsupported = Unsupported::parse(reader)?;
                    headers.push_header(Header::Unsupported(unsupported));
                }
                www_authenticate if WWWAuthenticate::match_name(www_authenticate) => {
                    let www_authenticate = WWWAuthenticate::parse(reader)?;
                    headers.push_header(Header::WWWAuthenticate(www_authenticate));
                }
                warning if Warning::match_name(warning) => {
                    let warning = Warning::parse(reader)?;
                    headers.push_header(Header::Warning(warning));
                }
                _ => {
                    let name = unsafe { str::from_utf8_unchecked(name) };
                    let value = until_newline!(reader);
                    let value = str::from_utf8(value)?;

                    headers.push_header(Header::Other { name, value });
                }
            };
            newline!(reader);
            if !reader.is_eof() {
                continue;
            }
            break 'headers;
        }

        Ok(())
    }
}
#[inline(always)]
pub(crate) fn is_host(b: u8) -> bool {
    HOST_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_user(b: u8) -> bool {
    USER_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_pass(b: u8) -> bool {
    PASS_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_param(b: u8) -> bool {
    PARAM_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_hdr(b: u8) -> bool {
    HDR_SPEC_MAP[b as usize]
}

#[inline(always)]
fn is_via_param(b: u8) -> bool {
    VIA_PARAM_SPEC_MAP[b as usize]
}

#[inline(always)]
pub(crate) fn is_uri_content(b: u8) -> bool {
    GENERIC_URI_SPEC_MAP[b as usize]
}

#[inline(always)]
pub(crate) fn is_token(b: u8) -> bool {
    TOKEN_SPEC_MAP[b as usize]
}

pub fn parse_sip_msg<'a>(buff: &'a [u8]) -> Result<SipMsg<'a>> {
    let mut parser = SipParser::new(buff);

    let msg = if parser.is_sip_request() {
        let req_line = parser.parse_request_line()?;
        let mut headers = SipHeaders::new();

        parser.parse_headers(&mut headers);

        todo!()
    } else {
        let status_line = parser.parse_status_line()?;
        let mut headers = SipHeaders::new();

        parser.parse_headers(&mut headers);

        todo!()
    };

    msg
}

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl SipParserError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl From<&str> for SipParserError {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<String> for SipParserError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl<'a> From<ReaderError<'a>> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{} column:{} kind:{:?}
                {}",
                err.line,
                err.col,
                err.kind,
                String::from_utf8_lossy(err.src)
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use crate::uri::Uri;

    use super::*;

    #[test]
    fn test_parse_status_line() {
        let sc_ok = SipStatusCode::Ok;
        let buf = "SIP/2.0 200 OK\r\n".as_bytes();

        assert_eq!(
            SipParser::new(buf).parse_status_line(),
            Ok(StatusLine {
                status_code: sc_ok,
                reason_phrase: sc_ok.reason_phrase()
            })
        );
        let sc_not_found = SipStatusCode::NotFound;
        let buf = "SIP/2.0 404 Not Found\r\n".as_bytes();

        assert_eq!(
            SipParser::new(buf).parse_status_line(),
            Ok(StatusLine {
                status_code: sc_not_found,
                reason_phrase: sc_not_found.reason_phrase()
            })
        );
    }

    #[test]
    fn status_line() {
        let sc_ok = SipStatusCode::Ok;
        let msg = "SIP/2.0 200 OK\r\n".as_bytes();
        let size_of_msg = msg.len();
        let mut counter = 0;
        let now = std::time::Instant::now();
        loop {
            assert_eq!(
                SipParser::new(msg).parse_status_line(),
                Ok(StatusLine {
                    status_code: sc_ok,
                    reason_phrase: sc_ok.reason_phrase()
                })
            );
            counter += 1;
            if now.elapsed().as_secs() == 1 {
                break;
            }
        }

        println!(
            "{} mbytes per second, count sip messages: {}",
            (size_of_msg * counter) / 1024 / 1024,
            counter
        );
    }

    #[test]
    fn test_req_status_line() {
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let addr: IpAddr = "10.1.1.7".parse().unwrap();
        assert_eq!(
            SipParser::new(msg).parse_request_line(),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: Uri {
                    scheme: Scheme::Sip,
                    user: Some(UserInfo {
                        name: "1000b3",
                        password: None
                    }),
                    host: HostPort::IpAddr {
                        host: addr,
                        port: Some(8089)
                    },
                    params: None,
                    other_params: None,
                    header_params: None,
                }
            })
        );
    }
}
