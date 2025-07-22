#![warn(missing_docs)]
//! SIP Parser
//!
//! This module contains functions for sip parsing.

use std::{
    borrow::Cow,
    str::{self},
};

use pksip_util::{
    util::{is_alphabetic, is_digit, is_newline, is_space, is_valid_port, not_comma_or_newline},
    Position, Scanner,
};

use crate::{
    error::Result,
    headers::*,
    macros::{b_map, comma_sep, parse_error, parse_header, parse_param},
    message::{
        auth::{Challenge, Credential, DigestChallenge, DigestCredential},
        Host, HostPort, NameAddr, Param, Params, Request, RequestLine, Response, Scheme, SipMethod, SipMsg, SipUri,
        StatusLine, Uri, UriUser,
    },
};

pub(crate) const SIPV2: &str = "SIP/2.0";
pub(crate) const CNONCE: &str = "cnonce";
pub(crate) const QOP: &str = "qop";
pub(crate) const NC: &str = "nc";
pub(crate) const NEXTNONCE: &str = "nextnonce";
pub(crate) const RSPAUTH: &str = "rspauth";

const B_SIPV2: &[u8] = SIPV2.as_bytes();
const USER_PARAM: &str = "user";
const METHOD_PARAM: &str = "method";
const TRANSPORT_PARAM: &str = "transport";
const TTL_PARAM: &str = "ttl";
const LR_PARAM: &str = "lr";
const MADDR_PARAM: &str = "maddr";
const SIP: &[u8] = b"sip";
const SIPS: &[u8] = b"sips";
const DIGEST: &str = "Digest";
const REALM: &str = "realm";
const USERNAME: &str = "username";
const NONCE: &str = "nonce";
const URI: &str = "uri";
const RESPONSE: &str = "response";
const ALGORITHM: &str = "algorithm";
const OPAQUE: &str = "opaque";
const DOMAIN: &str = "domain";
const STALE: &str = "stale";
const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const UNRESERVED: &[u8] = b"-_.!~*'()%";
const ESCAPED: &[u8] = b"%";
const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
const TOKEN: &[u8] = b"-.!%*_`'~+";
const PASS: &[u8] = b"&=+$,";
const HOST: &[u8] = b"_-.";

// For reading user part in uri.
b_map!(USER_MAP => ALPHA_NUM, UNRESERVED, USER_UNRESERVED, ESCAPED);
// For reading password  in uri.
b_map!(PASS_MAP => ALPHA_NUM, UNRESERVED, ESCAPED, PASS);
// For reading host in uri.
b_map!(HOST_MAP => ALPHA_NUM, HOST);
// For reading parameter in uri.
b_map!(PARAM_MAP => b"[]/:&+$", ALPHA_NUM, UNRESERVED, ESCAPED);
// For reading header parameter in uri.
b_map!(HDR_MAP => b"[]/?:+$", ALPHA_NUM, UNRESERVED, ESCAPED);
// For reading token.
b_map!(TOKEN_MAP => ALPHA_NUM, TOKEN);
// For reading via parameter.
b_map!(VIA_PARAM_MAP => b"[:]", ALPHA_NUM, TOKEN);

/// A type for parsing SIP messages.
///
/// This struct provides methods for parsing various components of SIP messages,
/// such as headers, URIs, and start lines.
pub struct Parser<'buf> {
    scanner: Scanner<'buf>,
}

impl<'buf> Parser<'buf> {
    /// Create an new `Parser` from the given slice.
    pub fn new(buf: &'buf [u8]) -> Self {
        Self {
            scanner: Scanner::new(buf),
        }
    }

    /// Parse a buffer of bytes into a `SipMsg`.
    ///
    /// # Example
    ///
    /// This example parses a simple SIP response message and asserts its contents:
    ///
    /// ```rust
    /// use pksip::parser::Parser;
    /// use pksip::headers::{Header, ContentLength};
    ///
    /// let buf = b"SIP/2.0 200 OK\r\nContent-Length: 0\r\n\r\n";
    /// let parser = &mut Parser::new(buf);
    /// let result = parser.parse_sip_msg().unwrap();
    /// let response = result.response().unwrap();
    /// assert_eq!(response.code().into_i32(), 200);
    /// assert_eq!(response.reason(), "OK");
    /// assert_eq!(response.headers.len(), 1);
    /// assert_eq!(response.headers[0], Header::ContentLength(0.into()));
    /// ```
    pub fn parse_sip_msg(&mut self) -> Result<SipMsg> {
        // Parse the start line of the SIP message and initialize the
        // message with empty headers and body.
        let mut msg = self.parse_start_line()?;
        let mut has_content_type = false;

        // Parse headers.
        let headers = msg.headers_mut();

        'headers: loop {
            // Get name.
            let name = self.parse_token()?;

            self.ws();

            let Some(b':') = self.advance() else {
                return self.parse_error("Missing ':' after header name");
            };

            self.ws();

            match name {
                ErrorInfo::NAME => {
                    let header = parse_header!(ErrorInfo, self);
                    headers.push(Header::ErrorInfo(header));
                }

                Route::NAME => comma_sep!(self => {
                    let header = parse_header!(Route, self);
                    headers.push(Header::Route(header));
                }),

                Via::NAME | Via::SHORT_NAME => comma_sep!(self => {
                    let header = parse_header!(Via, self);
                    headers.push(Header::Via(header));
                }),

                MaxForwards::NAME => {
                    let header = parse_header!(MaxForwards, self);
                    headers.push(Header::MaxForwards(header));
                }

                From::NAME | From::SHORT_NAME => {
                    let header = parse_header!(From, self);
                    headers.push(Header::From(header));
                }

                To::NAME | To::SHORT_NAME => {
                    let header = parse_header!(To, self);
                    headers.push(Header::To(header));
                }

                CallId::NAME | CallId::SHORT_NAME => {
                    let header = parse_header!(CallId, self);
                    headers.push(Header::CallId(header));
                }

                CSeq::NAME => {
                    let header = parse_header!(CSeq, self);
                    headers.push(Header::CSeq(header));
                }

                Authorization::NAME => {
                    let header = parse_header!(Authorization, self);
                    headers.push(Header::Authorization(header));
                }

                Contact::NAME | Contact::SHORT_NAME => comma_sep!(self => {
                    let header = parse_header!(Contact, self);
                    headers.push(Header::Contact(header));
                }),

                Expires::NAME => {
                    let header = parse_header!(Expires, self);
                    headers.push(Header::Expires(header));
                }

                InReplyTo::NAME => {
                    let header = parse_header!(InReplyTo, self);
                    headers.push(Header::InReplyTo(header));
                }

                MimeVersion::NAME => {
                    let header = parse_header!(MimeVersion, self);
                    headers.push(Header::MimeVersion(header));
                }

                MinExpires::NAME => {
                    let header = parse_header!(MinExpires, self);
                    headers.push(Header::MinExpires(header));
                }

                UserAgent::NAME => {
                    let header = parse_header!(UserAgent, self);
                    headers.push(Header::UserAgent(header));
                }

                Date::NAME => {
                    let header = parse_header!(Date, self);
                    headers.push(Header::Date(header));
                }

                Server::NAME => {
                    let header = parse_header!(Server, self);
                    headers.push(Header::Server(header));
                }

                Subject::NAME | Subject::SHORT_NAME => {
                    let header = parse_header!(Subject, self);
                    headers.push(Header::Subject(header));
                }

                Priority::NAME => {
                    let header = parse_header!(Priority, self);
                    headers.push(Header::Priority(header));
                }

                ProxyAuthenticate::NAME => {
                    let header = parse_header!(ProxyAuthenticate, self);
                    headers.push(Header::ProxyAuthenticate(header));
                }

                ProxyAuthorization::NAME => {
                    let header = parse_header!(ProxyAuthorization, self);
                    headers.push(Header::ProxyAuthorization(header));
                }

                ProxyRequire::NAME => {
                    let header = parse_header!(ProxyRequire, self);
                    headers.push(Header::ProxyRequire(header));
                }

                ReplyTo::NAME => {
                    let header = parse_header!(ReplyTo, self);
                    headers.push(Header::ReplyTo(header));
                }

                ContentLength::NAME | ContentLength::SHORT_NAME => {
                    let header = parse_header!(ContentLength, self);
                    headers.push(Header::ContentLength(header));
                }

                ContentEncoding::NAME | ContentEncoding::SHORT_NAME => {
                    let header = parse_header!(ContentEncoding, self);
                    headers.push(Header::ContentEncoding(header));
                }

                ContentType::NAME | ContentType::SHORT_NAME => {
                    let header = parse_header!(ContentType, self);
                    headers.push(Header::ContentType(header));
                    has_content_type = true;
                }

                ContentDisposition::NAME => {
                    let header = parse_header!(ContentDisposition, self);
                    headers.push(Header::ContentDisposition(header));
                }

                RecordRoute::NAME => comma_sep!(self => {
                    let header = parse_header!(RecordRoute, self);
                    headers.push(Header::RecordRoute(header));
                }),
                Require::NAME => {
                    let header = parse_header!(Require, self);
                    headers.push(Header::Require(header));
                }

                RetryAfter::NAME => {
                    let header = parse_header!(RetryAfter, self);
                    headers.push(Header::RetryAfter(header));
                }

                Organization::NAME => {
                    let header = parse_header!(Organization, self);
                    headers.push(Header::Organization(header));
                }

                AcceptEncoding::NAME => {
                    let header = parse_header!(AcceptEncoding, self);
                    headers.push(Header::AcceptEncoding(header));
                }

                Accept::NAME => {
                    let header = parse_header!(Accept, self);
                    headers.push(Header::Accept(header));
                }

                AcceptLanguage::NAME => {
                    let header = parse_header!(AcceptLanguage, self);
                    headers.push(Header::AcceptLanguage(header));
                }

                AlertInfo::NAME => {
                    let header = parse_header!(AlertInfo, self);
                    headers.push(Header::AlertInfo(header));
                }

                Allow::NAME => {
                    let header = parse_header!(Allow, self);
                    headers.push(Header::Allow(header));
                }

                AuthenticationInfo::NAME => {
                    let header = parse_header!(AuthenticationInfo, self);
                    headers.push(Header::AuthenticationInfo(header));
                }

                Supported::NAME | Supported::SHORT_NAME => {
                    let header = parse_header!(Supported, self);
                    headers.push(Header::Supported(header));
                }

                Timestamp::NAME => {
                    let header = parse_header!(Timestamp, self);
                    headers.push(Header::Timestamp(header));
                }
                Unsupported::NAME => {
                    let header = parse_header!(Unsupported, self);
                    headers.push(Header::Unsupported(header));
                }

                WWWAuthenticate::NAME => {
                    let header = parse_header!(WWWAuthenticate, self);
                    headers.push(Header::WWWAuthenticate(header));
                }

                Warning::NAME => {
                    let header = parse_header!(Warning, self);
                    headers.push(Header::Warning(header));
                }

                _ => {
                    // The header is not defined in rfc 3261.
                    let value = self.parse_header_str()?;

                    headers.push(Header::Other(OtherHeader { name, value }));
                }
            };

            if !matches!(self.peek(), Some(b'\r') | Some(b'\n')) {
                return self.parse_error("Missing CRLF on header end!");
            }

            // consumes empty lines.
            self.scanner.consume_if(|b| b == b'\r');
            self.scanner.consume_if(|b| b == b'\n');

            if matches!(self.peek(), Some(b'\r') | Some(b'\n') | None) {
                break 'headers;
            }
        }

        self.ws();

        if has_content_type {
            self.new_line();

            let rem = self.scanner.remaing();
            msg.set_body(Some(rem));
        }

        Ok(msg)
    }

    pub(crate) fn parse_error<T, S>(&self, msg: S) -> Result<T>
    where
        S: AsRef<str>,
    {
        parse_error!(msg.as_ref(), self.scanner)
    }

    pub(crate) fn parse_header_str(&mut self) -> Result<&'buf str> {
        let bytes = self.scanner.read_while(|b| !is_newline(b));

        Ok(str::from_utf8(bytes)?)
    }

    // Read whitespace characters.
    #[inline]
    pub(crate) fn ws(&mut self) {
        self.scanner.read_while(is_space);
    }

    // Read newline characters.
    #[inline]
    pub(crate) fn new_line(&mut self) {
        self.scanner.read_while(is_newline);
    }

    // Read alphabetic.
    #[inline]
    pub(crate) fn alphabetic(&mut self) -> &'buf [u8] {
        self.scanner.read_while(is_alphabetic)
    }

    // SIP version
    #[inline]
    pub(crate) fn parse_sip_v2(&mut self) -> Result<()> {
        Ok(self.scanner.matches_slice(B_SIPV2)?)
    }

    // SIP Request-Line.
    pub(crate) fn parse_request_line(&mut self) -> Result<RequestLine<'buf>> {
        let method_byte = self.alphabetic();
        let method = SipMethod::from(method_byte);

        self.ws();
        let uri = self.parse_uri(true)?;
        self.ws();

        self.parse_sip_v2()?;

        self.new_line();

        Ok(RequestLine { method, uri })
    }

    // SIP Status-Line.
    pub(crate) fn parse_status_line(&mut self) -> Result<StatusLine<'buf>> {
        self.parse_sip_v2()?;

        self.ws();
        let digits = self.scanner.read_while(is_digit);
        self.ws();

        let code = digits.into();

        let reason_byte = self.scanner.read_while(|b| !is_newline(b));
        let reason = str::from_utf8(reason_byte)?;

        self.new_line();

        Ok(StatusLine::new(code, reason))
    }

    fn parse_scheme(&mut self) -> Result<Scheme> {
        let (scheme_b, colon) = self.scanner.peek_while(is_token);

        let Some(b':') = colon else {
            return self.parse_error("Missing ':' in uri");
        };

        let scheme = match scheme_b {
            SIP => Scheme::Sip,
            SIPS => Scheme::Sips,
            scheme => return self.parse_error(format!("Unsupported URI scheme: {}", String::from_utf8_lossy(scheme))),
        };

        // Take the scheme and the character ":".
        self.scanner.bump_n(scheme_b.len() + 1);

        Ok(scheme)
    }

    fn exists_user_part_in_uri(&self) -> bool {
        let rem = self.scanner.remaing();

        rem.iter()
            .take_while(|&&b| b != b' ' && b != b'>' && !is_newline(b))
            .any(|&b| b == b'@')
    }

    // User info in SIP uri.
    pub(crate) fn parse_user_info(&mut self) -> Result<Option<UriUser<'buf>>> {
        // Checks if uri has an user part.
        let exists_user_in_uri = self.exists_user_part_in_uri();

        if !exists_user_in_uri {
            return Ok(None);
        }

        // We have user part in uri.
        let user = self.read_user_str().into();
        let pass = if let Some(b':') = self.scanner.consume_if(|b| b == b':') {
            Some(self.read_pass_str().into())
        } else {
            None
        };

        // Take '@'.
        self.advance();

        Ok(Some(UriUser { user, pass }))
    }

    pub(crate) fn parse_host_port(&mut self) -> Result<HostPort> {
        let host = match self.scanner.peek() {
            Some(b'[') => {
                // Is a Ipv6 host
                self.advance();
                // the '[' and ']' characters are removed from the host
                let host = self.scanner.read_while(|b| b != b']');
                let host = str::from_utf8(host)?;
                self.advance();

                match host.parse() {
                    Ok(addr) => Host::IpAddr(addr),
                    Err(_) => return self.parse_error("Error parsing Ipv6 HostPort!"),
                }
            }
            _ => {
                let host = self.read_host_str();

                if host.is_empty() {
                    return self.parse_error("Can't parse the host!");
                }
                match host.parse() {
                    Ok(addr) => Host::IpAddr(addr),
                    Err(_) => Host::DomainName(host.into()),
                }
            }
        };

        let port = self.parse_port()?;

        Ok(HostPort { host, port })
    }

    fn parse_port(&mut self) -> Result<Option<u16>> {
        let Some(b':') = self.scanner.consume_if(|b| b == b':') else {
            return Ok(None);
        };
        let digits = self.scanner.read_u16()?;

        if is_valid_port(digits) {
            Ok(Some(digits))
        } else {
            todo!("Sip Uri Port is invalid!")
        }
    }

    // Parse URI.
    pub(crate) fn parse_uri(&mut self, parse_params: bool) -> Result<Uri<'buf>> {
        let scheme = self.parse_scheme()?;
        let user = self.parse_user_info()?;
        let host_port = self.parse_host_port()?;

        if !parse_params {
            return Ok(Uri::without_params(scheme, user, host_port));
        }

        // Parse SIP uri parameters.
        let mut user_param = None;
        let mut method_param = None;
        let mut transport_param = None;
        let mut ttl_param = None;
        let mut lr_param = None;
        let mut maddr_param = None;

        let params = parse_param!(
            self,
            parse_uri_param,
            USER_PARAM = user_param,
            METHOD_PARAM = method_param,
            TRANSPORT_PARAM = transport_param,
            TTL_PARAM = ttl_param,
            LR_PARAM = lr_param,
            MADDR_PARAM = maddr_param
        );

        let transport_param = transport_param.map(|s| s.as_ref().into());
        let ttl_param = ttl_param.map(|ttl| ttl.parse().unwrap());
        let lr_param = lr_param.is_some();
        let method_param = method_param.map(|p| p.as_bytes().into());
        let user_param = user_param.map(|u| u.into());
        let maddr_param = maddr_param.map(|m| m.into());

        let hdr_params = if let Some(b'?') = self.scanner.consume_if(|b| b == b'?') {
            // The uri has header parameters.
            Some(self.parse_header_params_in_sip_uri()?)
        } else {
            None
        };

        Ok(Uri {
            scheme,
            user,
            host_port,
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

    fn parse_header_params_in_sip_uri(&mut self) -> Result<Params<'buf>> {
        let mut params = Params::new();

        loop {
            let param = self.parse_hdr_in_uri()?;
            params.push(param);

            if self.scanner.consume_if(|b| b == b'&').is_none() {
                break;
            }
        }
        Ok(params)
    }

    // Parse start line.
    fn parse_start_line(&mut self) -> Result<SipMsg<'buf>> {
        // Might be enough for most messages.
        let probable_number_of_headers = 10;

        if self.scanner.starts_with(B_SIPV2) {
            // Is an status line, e.g, "SIP/2.0 200 OK".
            let Ok(status_line) = self.parse_status_line() else {
                return self.parse_error("Error parsing 'Status Line'");
            };
            let headers = Headers::with_capacity(probable_number_of_headers);

            Ok(SipMsg::Response(Response {
                status_line,
                headers,
                body: None,
            }))
        } else {
            // Is an request line, e.g, "OPTIONS sip:localhost SIP/2.0".
            let Ok(req_line) = self.parse_request_line() else {
                return self.parse_error("Error parsing 'Request Line'");
            };
            let headers = Headers::with_capacity(probable_number_of_headers);

            Ok(SipMsg::Request(Request {
                req_line,
                headers,
                body: None,
            }))
        }
    }

    fn parse_display_name(&mut self) -> Result<Option<&'buf str>> {
        match self.scanner.lookahead()? {
            b'"' => {
                self.advance(); // consume '"'
                let name = self.scanner.read_while(|b| b != b'"');
                self.advance(); // consume closing '"'
                Ok(Some(str::from_utf8(name)?))
            }
            b'<' => Ok(None), // no display name
            _ => {
                let name = self.parse_token()?;
                self.ws();
                Ok(Some(name))
            }
        }
    }

    #[inline]
    pub(crate) fn parse_token(&mut self) -> Result<&'buf str> {
        if let Some(b'"') = self.scanner.consume_if(|b| b == b'"') {
            let value = self.scanner.read_while(|b| b != b'"');
            self.advance();

            Ok(str::from_utf8(value)?)
        } else {
            // is_token ensures that is valid UTF-8
            Ok(self.read_token_str())
        }
    }

    // Parse SIP Uri.
    pub(crate) fn parse_sip_uri(&mut self, parse_params: bool) -> Result<SipUri<'buf>> {
        self.ws();

        match self.scanner.peek_n(3) {
            Some(SIP) | Some(SIPS) => {
                let uri = self.parse_uri(parse_params)?;
                Ok(SipUri::Uri(uri))
            }
            _ => {
                let addr = self.parse_name_addr()?;
                Ok(SipUri::NameAddr(addr))
            }
        }
    }

    #[inline]
    pub(crate) fn advance(&mut self) -> Option<u8> {
        self.scanner.next()
    }

    #[inline]
    pub(crate) fn read_until_byte(&mut self, byte: u8) -> &'buf [u8] {
        self.scanner.take_until(byte)
    }

    #[inline]
    pub(crate) fn peek(&self) -> Option<&u8> {
        self.scanner.peek()
    }

    #[inline]
    pub(crate) fn position(&self) -> &Position {
        self.scanner.position()
    }

    #[inline]
    pub(crate) fn remaing(&self) -> &[u8] {
        self.scanner.remaing()
    }

    #[inline]
    pub(crate) fn not_comma_or_newline(&mut self) -> &'buf [u8] {
        self.scanner.read_while(not_comma_or_newline)
    }

    #[inline]
    pub(crate) fn is_next_newline(&self) -> bool {
        self.scanner.peek().is_some_and(|&b| is_newline(b))
    }

    #[inline]
    pub(crate) fn parse_u32(&mut self) -> Result<u32> {
        Ok(self.scanner.read_u32()?)
    }

    #[inline]
    pub(crate) fn must_read(&mut self, b: u8) -> Result<()> {
        Ok(self.scanner.must_read(b)?)
    }

    #[inline]
    pub(crate) fn parse_number_str(&mut self) -> &'buf str {
        self.scanner.scan_number_str()
    }

    #[inline]
    pub(crate) fn parse_num<N: lexical_core::FromLexical>(&mut self) -> Result<N> {
        Ok(self.scanner.read_num()?)
    }

    pub(crate) fn parse_name_addr(&mut self) -> Result<NameAddr<'buf>> {
        self.ws();
        let display = self.parse_display_name()?;
        self.ws();

        // must be an '<'
        let Some(b'<') = self.scanner.next() else {
            return self.parse_error("Expected '<' in NameAddr!");
        };

        let uri = self.parse_uri(true)?;

        // must be an '>'
        let Some(b'>') = self.scanner.next() else {
            return self.parse_error("Expected '>' in NameAddr!");
        };

        Ok(NameAddr {
            display: display.map(Cow::Borrowed),
            uri,
        })
    }

    #[inline]
    pub(crate) unsafe fn read_as_str(&mut self, func: impl Fn(u8) -> bool) -> &'buf str {
        self.scanner.read_as_str(func)
    }

    #[inline]
    fn read_user_str(&mut self) -> &'buf str {
        unsafe { self.read_as_str(is_user) }
    }

    #[inline]
    fn read_pass_str(&mut self) -> &'buf str {
        unsafe { self.read_as_str(is_pass) }
    }

    #[inline]
    fn read_host_str(&mut self) -> &'buf str {
        unsafe { self.read_as_str(is_host) }
    }

    #[inline]
    fn read_token_str(&mut self) -> &'buf str {
        unsafe { self.read_as_str(is_token) }
    }

    pub(crate) unsafe fn parse_param_unchecked<F>(&mut self, func: F) -> Result<Param<'buf>>
    where
        F: Fn(u8) -> bool,
    {
        self.ws();

        let name = unsafe { self.scanner.read_as_str(&func) };

        let Some(b'=') = self.scanner.peek() else {
            return Ok(Param {
                name: name.into(),
                value: None,
            });
        };

        self.advance();

        let value = if let Some(b'"') = self.scanner.peek() {
            self.advance();
            let value = self.scanner.read_while(|b| b != b'"');
            self.advance();

            str::from_utf8(value)?
        } else {
            unsafe { self.scanner.read_as_str(func) }
        };

        Ok(Param {
            name: name.into(),
            value: Some(value.into()),
        })
    }

    // Parse parameter (";" pname ["=" pvalue]).
    pub(crate) fn parse_param(&mut self) -> Result<Param<'buf>> {
        unsafe { self.parse_param_unchecked(is_token) }
    }

    pub(crate) fn parse_auth_credential(&mut self) -> Result<Credential<'buf>> {
        let scheme = self.parse_token()?;

        if scheme == DIGEST {
            return self.parse_digest_credential();
        }

        self.parse_other_credential(scheme)
    }

    pub(crate) fn parse_auth_challenge(&mut self) -> Result<Challenge<'buf>> {
        let scheme = self.parse_token()?;

        if scheme == DIGEST {
            return self.parse_digest_challenge();
        }

        let mut params = Params::new();

        comma_sep!(self => {
            let param = self.parse_param()?;

            params.push(param);

        });

        Ok(Challenge::Other {
            scheme: scheme.into(),
            param: params,
        })
    }

    fn parse_digest_challenge(&mut self) -> Result<Challenge<'buf>> {
        let mut digest = DigestChallenge::default();

        comma_sep!(self => {
            let Param {name, value} = self.parse_param()?;

            match name.as_ref() {
                REALM => digest.realm = value,
                NONCE => digest.nonce = value,
                DOMAIN => digest.domain = value,
                ALGORITHM => digest.algorithm = value,
                OPAQUE => digest.opaque = value,
                QOP => digest.qop = value,
                STALE => digest.stale = value,
                _other => {
                    // return err?
                }
            }
        });

        Ok(Challenge::Digest(digest))
    }

    fn parse_digest_credential(&mut self) -> Result<Credential<'buf>> {
        let mut digest = DigestCredential::default();

        comma_sep!(self => {
            let Param { name, value } = self.parse_param()?;
            match name.as_ref() {
                REALM => digest.realm = value,
                USERNAME => digest.username = value,
                NONCE => digest.nonce = value,
                URI => digest.uri = value,
                RESPONSE => digest.response = value,
                ALGORITHM => digest.algorithm = value,
                CNONCE => digest.cnonce = value,
                OPAQUE => digest.opaque = value,
                QOP => digest.qop = value,
                NC => digest.nc = value,
                _ => {}, // Ignore unknown parameters
            }
        });

        Ok(Credential::Digest(digest))
    }

    fn parse_other_credential(&mut self, scheme: &'buf str) -> Result<Credential<'buf>> {
        let mut param = Params::new();

        comma_sep!(self => {
            let mut p = self.parse_param()?;

            if p.value.is_none() {
                p.value = Some("".into());
            }

            param.push(p);
        });

        Ok(Credential::Other {
            scheme: scheme.into(),
            param,
        })
    }

    #[inline]
    fn parse_hdr_in_uri(&mut self) -> Result<Param<'buf>> {
        // SAFETY: `is_hdr_uri` only accepts ASCII bytes, which are always
        // valid UTF-8.
        Ok(unsafe { self.parse_param_unchecked(is_hdr_uri)? })
    }
}

#[inline(always)]
fn is_user(b: u8) -> bool {
    USER_MAP[b as usize]
}

#[inline(always)]
fn is_pass(b: u8) -> bool {
    PASS_MAP[b as usize]
}

#[inline(always)]
fn is_param(b: u8) -> bool {
    PARAM_MAP[b as usize]
}

#[inline(always)]
fn is_hdr_uri(b: u8) -> bool {
    HDR_MAP[b as usize]
}

#[inline(always)]
pub(crate) fn is_host(b: u8) -> bool {
    HOST_MAP[b as usize]
}

#[inline(always)]
pub(crate) fn is_token(b: u8) -> bool {
    TOKEN_MAP[b as usize]
}

#[inline(always)]
pub(crate) fn is_via_param(b: u8) -> bool {
    VIA_PARAM_MAP[b as usize]
}

#[inline]
pub(crate) fn parse_via_param<'a>(parser: &mut Parser<'a>) -> Result<Param<'a>> {
    // SAFETY: `is_via_param` only accepts ASCII bytes, which are always
    // valid UTF-8.
    unsafe { parser.parse_param_unchecked(is_via_param) }
}

fn parse_uri_param<'a>(parser: &mut Parser<'a>) -> Result<Param<'a>> {
    // SAFETY: `is_param` only accepts ASCII bytes, which are always
    // valid UTF-8.
    let mut param = unsafe { parser.parse_param_unchecked(is_param)? };

    if param.name == LR_PARAM && param.value.is_none() {
        param.value = Some("".into());
    }

    Ok(param)
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    #[test]
    fn test_uri_1() {
        let src = "sip:bob@biloxi.com";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_2() {
        let src = "sip:bob@192.0.2.201";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "192.0.2.201");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_3() {
        let src = "sip:bob@[2620:0:2ef0:7070:250:60ff:fe03:32b7]";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "2620:0:2ef0:7070:250:60ff:fe03:32b7");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_4() {
        let src = "sip:bob:pass@biloxi.com";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, Some(Cow::Borrowed("pass")));
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_5() {
        let src = "sip:bob:pass@192.0.2.201";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, Some(Cow::Borrowed("pass")));
        assert!(parsed.host_port().is_ip_addr());
        assert_eq!(parsed.host_port().host_as_str(), "192.0.2.201");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_6() {
        let src = "sip:biloxi.com";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert!(parsed.user().is_none());
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
    }

    #[test]
    fn test_uri_7() {
        let src = "sip:bob@biloxi.com:5060";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
    }

    #[test]
    fn test_uri_8() {
        let src = "sip:bob:pass@biloxi.com:5060";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, Some("pass".into()));
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
    }

    #[test]
    fn test_uri_9() {
        let src = "sip:bob@biloxi.com;foo=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
        assert_eq!(parsed.params().unwrap().get("foo").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_uri_10() {
        let src = "sip:bob@biloxi.com:5060;foo=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.params().unwrap().get("foo").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_uri_11() {
        let src = "sip:bob@biloxi.com:5060;foo";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.params().unwrap().get("foo"), Some(None));
    }

    #[test]
    fn test_uri_12() {
        let src = "sip:bob@biloxi.com:5060;foo;baz=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.params().unwrap().get("foo"), Some(None));
        assert_eq!(parsed.params().unwrap().get("baz").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_uri_13() {
        let src = "sip:bob@biloxi.com:5060;baz=bar;foo";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.params().unwrap().get("baz").unwrap().unwrap(), "bar");
        assert_eq!(parsed.params().unwrap().get("foo"), Some(None));
    }

    #[test]
    fn test_uri_14() {
        let src = "sip:bob@biloxi.com:5060;baz=bar;foo;a=b";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.params().unwrap().get("baz").unwrap().unwrap(), "bar");
        assert_eq!(parsed.params().unwrap().get("foo"), Some(None));
        assert_eq!(parsed.params().unwrap().get("a").unwrap().unwrap(), "b");
    }

    #[test]
    fn test_uri_15() {
        let src = "sip:bob@biloxi.com?foo=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
        assert_eq!(parsed.header_params().unwrap().get("foo").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_uri_16() {
        let src = "sip:bob@biloxi.com?foo";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
        assert_eq!(parsed.header_params().unwrap().get("foo"), Some(None));
    }

    #[test]
    fn test_uri_17() {
        let src = "sip:bob@biloxi.com:5060?foo=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.header_params().unwrap().get("foo").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_uri_18() {
        let src = "sip:bob@biloxi.com:5060?baz=bar&foo=&a=b";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.header_params().unwrap().get("baz").unwrap().unwrap(), "bar");
        assert_eq!(parsed.header_params().unwrap().get("foo"), Some(Some("")));
        assert_eq!(parsed.header_params().unwrap().get("a").unwrap().unwrap(), "b");
    }

    #[test]
    fn test_uri_19() {
        let src = "sip:bob@biloxi.com:5060?foo=bar&baz";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, Some(5060));
        assert_eq!(parsed.header_params().unwrap().get("foo").unwrap().unwrap(), "bar");
        assert_eq!(parsed.header_params().unwrap().get("baz"), Some(None));
    }

    #[test]
    fn test_uri_20() {
        let src = "sip:bob@biloxi.com;foo?foo=bar";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_sip_uri(true).unwrap();

        assert_eq!(parsed.scheme(), Scheme::Sip);
        assert_eq!(parsed.user().unwrap().user, "bob");
        assert_eq!(parsed.user().unwrap().pass, None);
        assert_eq!(parsed.host_port().host_as_str(), "biloxi.com");
        assert_eq!(parsed.host_port().port, None);
        assert_eq!(parsed.params().unwrap().get("foo"), Some(None));
        assert_eq!(parsed.header_params().unwrap().get("foo").unwrap().unwrap(), "bar");
    }

    #[test]
    fn test_host_port_1() {
        let src = "example.com";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_host_port().unwrap();

        assert_eq!(parsed.host, Host::DomainName("example.com".into()));
        assert_eq!(parsed.port, None);
    }

    #[test]
    fn test_host_port_2() {
        let src = "192.0.2.201";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_host_port().unwrap();

        assert_eq!(parsed.host, Host::IpAddr("192.0.2.201".parse().unwrap()));
        assert_eq!(parsed.port, None);
    }

    #[test]
    fn test_host_port_3() {
        let src = "example.com:5060";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_host_port().unwrap();

        assert_eq!(parsed.host, Host::DomainName("example.com".into()));
        assert_eq!(parsed.port, Some(5060));
    }

    #[test]
    fn test_host_port_4() {
        let src = "192.0.2.201:5060";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_host_port().unwrap();

        assert_eq!(parsed.host, Host::IpAddr("192.0.2.201".parse().unwrap()));
        assert_eq!(parsed.port, Some(5060));
    }

    #[test]
    fn test_host_port_5() {
        let src = "[2620:0:2ef0:7070:250:60ff:fe03:32b7]:5060";
        let parser = &mut Parser::new(src.as_bytes());

        let parsed = parser.parse_host_port().unwrap();

        assert_eq!(
            parsed.host,
            Host::IpAddr("2620:0:2ef0:7070:250:60ff:fe03:32b7".parse().unwrap())
        );
        assert_eq!(parsed.port, Some(5060));
    }
    #[test]
    fn test_parse_request_without_body() {
        let raw_msg = concat!(
            "INVITE sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@example.com>\r\n",
            "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Content-Length: 0\r\n\r\n"
        );

        let mut parser = Parser::new(raw_msg.as_bytes());
        let sip_msg = parser.parse_sip_msg().unwrap();
        let request = sip_msg.request().unwrap();

        let expected_uri = Uri::from_static("sip:bob@example.com").unwrap();
        let expected_headers = crate::headers![
            Header::Via(Via::new_udp(
                "pc33.atlanta.com".parse().unwrap(),
                Some("z9hG4bK776asdhds")
            )),
            Header::MaxForwards(MaxForwards::new(70)),
            Header::To(To::from_str("Bob <sip:bob@example.com>").unwrap()),
            Header::From(From::from_str("Alice <sip:alice@example.com>;tag=1928301774").unwrap()),
            Header::CallId(CallId::new("a84b4c76e66710")),
            Header::CSeq(CSeq::new(314159, SipMethod::Invite)),
            Header::ContentLength(ContentLength::default()),
        ];

        assert_eq!(request.method(), &SipMethod::Invite);
        assert_eq!(request.req_line.uri, expected_uri);
        assert_eq!(request.headers, expected_headers);
        assert_eq!(request.body, None);
    }

    #[test]
    fn test_parse_request_with_body() {
        let raw_msg = concat!(
            "INVITE sip:bob@biloxi.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@biloxi.com>\r\n",
            "From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Contact: <sip:alice@pc33.atlanta.com>\r\n",
            "Content-Type: application/sdp\r\n",
            "Content-Length: 4\r\n",
            "\r\n",
            "Test\r\n",
        );

        let mut parser = Parser::new(raw_msg.as_bytes());
        let sip_msg = parser.parse_sip_msg().unwrap();
        let request = sip_msg.request().unwrap();

        let expected_uri = Uri::from_static("sip:bob@biloxi.com").unwrap();
        let expected_headers = crate::headers![
            Header::Via(Via::new_udp(
                "pc33.atlanta.com".parse().unwrap(),
                Some("z9hG4bK776asdhds")
            )),
            Header::MaxForwards(MaxForwards::new(70)),
            Header::To(To::from_str("Bob <sip:bob@biloxi.com>").unwrap()),
            Header::From(From::from_str("Alice <sip:alice@atlanta.com>;tag=1928301774").unwrap()),
            Header::CallId(CallId::new("a84b4c76e66710@pc33.atlanta.com")),
            Header::CSeq(CSeq::new(314159, SipMethod::Invite)),
            Header::Contact(Contact::from_str("<sip:alice@pc33.atlanta.com>").unwrap()),
            Header::ContentType(ContentType::new_sdp()),
            Header::ContentLength(ContentLength::new(4)),
        ];

        assert_eq!(request.method(), &SipMethod::Invite);
        assert_eq!(request.req_line.uri, expected_uri);
        assert_eq!(request.headers, expected_headers);
        assert_eq!(request.body, Some("Test\r\n".as_bytes()));
    }

    #[test]
    fn test_parse_response_without_body() {
        let raw_msg = concat!(
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@example.com>\r\n",
            "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Content-Length: 0\r\n\r\n"
        );

        let mut parser = Parser::new(raw_msg.as_bytes());
        let msg = parser.parse_sip_msg().unwrap();
        let msg = msg.response().unwrap();

        let expected_headers = crate::headers![
            Header::Via(Via::new_udp(
                "pc33.atlanta.com".parse().unwrap(),
                Some("z9hG4bK776asdhds")
            )),
            Header::MaxForwards(MaxForwards::new(70)),
            Header::To(To::from_str("Bob <sip:bob@example.com>").unwrap()),
            Header::From(From::from_str("Alice <sip:alice@example.com>;tag=1928301774").unwrap()),
            Header::CallId(CallId::new("a84b4c76e66710")),
            Header::CSeq(CSeq::new(314159, SipMethod::Invite)),
            Header::ContentLength(ContentLength::default()),
        ];

        assert_eq!(msg.code().into_i32(), 200);
        assert_eq!(msg.reason(), "OK");
        assert_eq!(msg.headers, expected_headers);
        assert_eq!(msg.body, None);
    }

    #[test]
    fn test_parse_request_with_invalid_uri() {
        let raw_msg = concat!(
            "INVITE bob@biloxi.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@biloxi.com>\r\n",
            "From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Contact: <sip:alice@pc33.atlanta.com>\r\n",
            "Content-Type: application/sdp\r\n",
            "Content-Length: 4\r\n",
            "\r\n",
            "Test\r\n",
        );

        let mut parser = Parser::new(raw_msg.as_bytes());

        assert!(parser.parse_sip_msg().is_err());
    }
}
