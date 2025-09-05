//! SIP Parser
//!
//! The module provides [`Parser`] struct for parsing SIP messages, including
//! requests and responses, as well as various components such as URIs and
//! headers.

use std::str::{self};

use util::{Position, Scanner};

use crate::header::*;
use crate::macros::{comma_separated, lookup_table, parse_error, parse_param, try_parse_hdr};
use crate::message::*;
use crate::Result;

// ---------------------------------------------------------------------
// Parser constants
// ---------------------------------------------------------------------
/// The user param used in SIP URIs.
const USER_PARAM: &str = "user";
/// The method param used in SIP URIs.
const METHOD_PARAM: &str = "method";
/// The transport param used in SIP URIs.
const TRANSPORT_PARAM: &str = "transport";
/// The ttl param used in SIP URIs.
const TTL_PARAM: &str = "ttl";
/// The lr param used in SIP URIs.
const LR_PARAM: &str = "lr";
/// The maddr param used in SIP URIs.
const MADDR_PARAM: &str = "maddr";
/// Alphanumeric is valid in all sip message components.
const ALPHANUMERIC: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
/// Unreserved characters in user, password, uri and header
/// parameters in SIP uris.
const UNRESERVED: &[u8] = b"-_.!~*'()%";
/// Escaped character in SIP URIs.
const ESCAPED: &[u8] = b"%";
/// Unreserverd charaters in user part of SIP URIs.
const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
/// Token in SIP Messages
const TOKEN: &[u8] = b"-.!%*_`'~+";
/// Password valid characters in SIP URIs.
const PASS: &[u8] = b"&=+$,";
/// Valid characters in SIP URIs host part.
const HOST: &[u8] = b"_-.";
/// The "sip" schema used in SIP URIs.
const SIP: &[u8] = b"sip";
/// The "sips" schema used in SIP URIs.
const SIPS: &[u8] = b"sips";
/// The SIP version used in the parser.
pub(crate) const SIPV2: &str = "SIP/2.0";

const B_SIPV2: &[u8] = SIPV2.as_bytes();

// ---------------------------------------------------------------------
// Lookup Tables
// ---------------------------------------------------------------------
// For reading user in uri.
lookup_table!(USER_TAB => ALPHANUMERIC, UNRESERVED, USER_UNRESERVED, ESCAPED);
// For reading password in uri.
lookup_table!(PASS_TAB => ALPHANUMERIC, UNRESERVED, ESCAPED, PASS);
// For reading host in uri.
lookup_table!(HOST_TAB => ALPHANUMERIC, HOST);
// For reading parameter in uri.
lookup_table!(PARAM_TAB => b"[]/:&+$", ALPHANUMERIC, UNRESERVED, ESCAPED);
// For reading header parameter in uri.
lookup_table!(HDR_TAB => b"[]/?:+$", ALPHANUMERIC, UNRESERVED, ESCAPED);
// For reading token.
lookup_table!(TOKEN_TAB => ALPHANUMERIC, TOKEN);
// For reading via parameter.
lookup_table!(VIA_PARAM_TAB => b"[:]", ALPHANUMERIC, TOKEN);

type ParamRef<'a> = (&'a str, Option<&'a str>);

/// A SIP message parser.
///
/// This struct provides methods for parsing various components of SIP messages,
/// such as header fields, URIs, and start lines.
pub struct Parser<'buf> {
    /// The scanner used to read the input buffer.
    scanner: Scanner<'buf>,
}

impl<'buf> Parser<'buf> {
    /// Creates a new `Parser` from the given byte slice.
    #[inline]
    pub fn new<B>(buf: &'buf B) -> Self
    where
        B: AsRef<[u8]> + ?Sized,
    {
        Self {
            scanner: Scanner::new(buf.as_ref()),
        }
    }

    /// Parses the `buf` into a [`SipMessage`].
    ///
    /// This is equivalent to `Parser::new(buf).parse()`.
    #[inline]
    pub fn parse_sip_msg<B>(buf: &'buf B) -> Result<SipMessage>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        Self::new(buf.as_ref()).parse()
    }

    /// Parses the internal buffer into a [`SipMessage`].
    ///
    /// # Examples
    ///
    /// ```
    /// let buf = b"SIP/2.0 200 OK\r\nContent-Length: 0\r\n\r\n";
    /// let msg = Parser::new().parse(buf).unwrap();
    /// let res = result.response().unwrap();
    ///
    /// assert_eq!(res.code().as_u16(), 200);
    /// assert_eq!(res.reason(), "OK");
    /// assert_eq!(res.headers.len(), 1);
    /// ```
    pub fn parse(&mut self) -> Result<SipMessage> {
        let mut sip_message = self.parse_start_line()?;

        let mut has_content_type = false;
        // Parse headers loop.
        let headers = sip_message.headers_mut();
        'headers: loop {
            // Get name.
            let header_name = self.parse_token()?;

            self.space();
            self.must_read(b':')?;
            self.space();

            match header_name {
                ErrorInfo::NAME => {
                    let header = try_parse_hdr!(ErrorInfo, self);
                    headers.push(Header::ErrorInfo(header));
                }
                Route::NAME => comma_separated!(self => {
                    let header = try_parse_hdr!(Route, self);
                    headers.push(Header::Route(header));
                }),
                Via::NAME | Via::SHORT_NAME => comma_separated!(self => {
                    let header = try_parse_hdr!(Via, self);
                    headers.push(Header::Via(header));
                }),
                MaxForwards::NAME => {
                    let header = try_parse_hdr!(MaxForwards, self);
                    headers.push(Header::MaxForwards(header));
                }
                From::NAME | From::SHORT_NAME => {
                    let header = try_parse_hdr!(From, self);
                    headers.push(Header::From(header));
                }
                To::NAME | To::SHORT_NAME => {
                    let header = try_parse_hdr!(To, self);
                    headers.push(Header::To(header));
                }
                CallId::NAME | CallId::SHORT_NAME => {
                    let header = try_parse_hdr!(CallId, self);
                    headers.push(Header::CallId(header));
                }
                CSeq::NAME => {
                    let header = try_parse_hdr!(CSeq, self);
                    headers.push(Header::CSeq(header));
                }
                Authorization::NAME => {
                    let header = try_parse_hdr!(Authorization, self);
                    headers.push(Header::Authorization(header));
                }
                Contact::NAME | Contact::SHORT_NAME => comma_separated!(self => {
                    let header = try_parse_hdr!(Contact, self);
                    headers.push(Header::Contact(header));
                }),
                Expires::NAME => {
                    let header = try_parse_hdr!(Expires, self);
                    headers.push(Header::Expires(header));
                }
                InReplyTo::NAME => {
                    let header = try_parse_hdr!(InReplyTo, self);
                    headers.push(Header::InReplyTo(header));
                }
                MimeVersion::NAME => {
                    let header = try_parse_hdr!(MimeVersion, self);
                    headers.push(Header::MimeVersion(header));
                }
                MinExpires::NAME => {
                    let header = try_parse_hdr!(MinExpires, self);
                    headers.push(Header::MinExpires(header));
                }
                UserAgent::NAME => {
                    let header = try_parse_hdr!(UserAgent, self);
                    headers.push(Header::UserAgent(header));
                }
                Date::NAME => {
                    let header = try_parse_hdr!(Date, self);
                    headers.push(Header::Date(header));
                }
                Server::NAME => {
                    let header = try_parse_hdr!(Server, self);
                    headers.push(Header::Server(header));
                }
                Subject::NAME | Subject::SHORT_NAME => {
                    let header = try_parse_hdr!(Subject, self);
                    headers.push(Header::Subject(header));
                }
                Priority::NAME => {
                    let header = try_parse_hdr!(Priority, self);
                    headers.push(Header::Priority(header));
                }
                ProxyAuthenticate::NAME => {
                    let header = try_parse_hdr!(ProxyAuthenticate, self);
                    headers.push(Header::ProxyAuthenticate(header));
                }
                ProxyAuthorization::NAME => {
                    let header = try_parse_hdr!(ProxyAuthorization, self);
                    headers.push(Header::ProxyAuthorization(header));
                }
                ProxyRequire::NAME => {
                    let header = try_parse_hdr!(ProxyRequire, self);
                    headers.push(Header::ProxyRequire(header));
                }
                ReplyTo::NAME => {
                    let header = try_parse_hdr!(ReplyTo, self);
                    headers.push(Header::ReplyTo(header));
                }
                ContentLength::NAME | ContentLength::SHORT_NAME => {
                    let header = try_parse_hdr!(ContentLength, self);
                    headers.push(Header::ContentLength(header));
                }
                ContentEncoding::NAME | ContentEncoding::SHORT_NAME => {
                    let header = try_parse_hdr!(ContentEncoding, self);
                    headers.push(Header::ContentEncoding(header));
                }
                ContentType::NAME | ContentType::SHORT_NAME => {
                    let header = try_parse_hdr!(ContentType, self);
                    headers.push(Header::ContentType(header));
                    has_content_type = true;
                }
                ContentDisposition::NAME => {
                    let header = try_parse_hdr!(ContentDisposition, self);
                    headers.push(Header::ContentDisposition(header));
                }
                RecordRoute::NAME => comma_separated!(self => {
                    let header = try_parse_hdr!(RecordRoute, self);
                    headers.push(Header::RecordRoute(header));
                }),
                Require::NAME => {
                    let header = try_parse_hdr!(Require, self);
                    headers.push(Header::Require(header));
                }
                RetryAfter::NAME => {
                    let header = try_parse_hdr!(RetryAfter, self);
                    headers.push(Header::RetryAfter(header));
                }
                Organization::NAME => {
                    let header = try_parse_hdr!(Organization, self);
                    headers.push(Header::Organization(header));
                }
                AcceptEncoding::NAME => {
                    let header = try_parse_hdr!(AcceptEncoding, self);
                    headers.push(Header::AcceptEncoding(header));
                }
                Accept::NAME => {
                    let header = try_parse_hdr!(Accept, self);
                    headers.push(Header::Accept(header));
                }
                AcceptLanguage::NAME => {
                    let header = try_parse_hdr!(AcceptLanguage, self);
                    headers.push(Header::AcceptLanguage(header));
                }
                AlertInfo::NAME => {
                    let header = try_parse_hdr!(AlertInfo, self);
                    headers.push(Header::AlertInfo(header));
                }
                Allow::NAME => {
                    let header = try_parse_hdr!(Allow, self);
                    headers.push(Header::Allow(header));
                }
                AuthenticationInfo::NAME => {
                    let header = try_parse_hdr!(AuthenticationInfo, self);
                    headers.push(Header::AuthenticationInfo(header));
                }
                Supported::NAME | Supported::SHORT_NAME => {
                    let header = try_parse_hdr!(Supported, self);
                    headers.push(Header::Supported(header));
                }
                Timestamp::NAME => {
                    let header = try_parse_hdr!(Timestamp, self);
                    headers.push(Header::Timestamp(header));
                }
                Unsupported::NAME => {
                    let header = try_parse_hdr!(Unsupported, self);
                    headers.push(Header::Unsupported(header));
                }
                WWWAuthenticate::NAME => {
                    let header = try_parse_hdr!(WWWAuthenticate, self);
                    headers.push(Header::WWWAuthenticate(header));
                }
                Warning::NAME => {
                    let header = try_parse_hdr!(Warning, self);
                    headers.push(Header::Warning(header));
                }
                name => {
                    // Found a header that is not defined in RFC 3261.
                    let data = self.read_until_new_line()?;
                    let header = RawHeader::new(name, data);
                    headers.push(Header::RawHeader(header));
                }
            };

            if self.advance_if_eq(b'\r').is_none() || self.advance_if_eq(b'\n').is_none() {
                return self.parse_error("Missing CRLF on header end!".into());
            }

            if matches!(self.peek_byte(), Some(b'\r') | Some(b'\n') | None) {
                break 'headers;
            }
        }

        self.space();

        if has_content_type {
            self.new_line();

            let body = self.remaining();
            sip_message.set_body(body.into());
        }

        Ok(sip_message)
    }

    fn parse_start_line(&mut self) -> Result<SipMessage> {
        // Might be enough for most messages.
        let probable_number_of_headers = 10;

        if matches!(self.scanner.peek_bytes(B_SIPV2.len()), Some(B_SIPV2)) {
            // Is an status line, e.g, "SIP/2.0 200 OK".
            // TODO: Add "match" here.
            let status_line = self.parse_status_line()?;
            let headers = Headers::with_capacity(probable_number_of_headers);

            Ok(SipMessage::Response(Response {
                status_line,
                headers,
                body: None,
            }))
        } else {
            // Is an request line, e.g, "OPTIONS sip:localhost SIP/2.0".
            // TODO: Add "match" here.
            let req_line = self.parse_request_line()?;
            let headers = Headers::with_capacity(probable_number_of_headers);

            Ok(SipMessage::Request(Request {
                req_line,
                headers,
                body: None,
            }))
        }
    }

    fn parse_status_line(&mut self) -> Result<StatusLine> {
        self.parse_sip_version()?;
        let code = self.parse_status_code()?;
        let reason = self.read_until_new_line()?.into();
        self.new_line();

        Ok(StatusLine { code, reason })
    }

    fn parse_request_line(&mut self) -> Result<RequestLine> {
        let token = self.read_while(is_token);
        let method = token.into();
        let uri = self.parse_uri(true)?;
        self.parse_sip_version()?;

        self.new_line();

        Ok(RequestLine { method, uri })
    }

    #[inline]
    pub(crate) fn parse_sip_version(&mut self) -> Result<()> {
        self.must_read_bytes(B_SIPV2)
    }

    fn parse_status_code(&mut self) -> Result<StatusCode> {
        self.space();
        let digits = self.read_digits();
        self.space();

        let code = digits
            .try_into()
            .or_else(|_| self.parse_error("Invalid status code".into()))?;

        Ok(code)
    }

    fn parse_scheme(&mut self) -> Result<Scheme> {
        let token = self.scanner.peek_while(is_token);

        let scheme = match token {
            SIP => Scheme::Sip,
            SIPS => Scheme::Sips,
            other => {
                return self.parse_error(format!(
                    "Unsupported URI scheme: {}",
                    String::from_utf8_lossy(other)
                ))
            }
        };

        // Eat the scheme.
        self.scanner.advance_by(token.len());
        // Eat the ":" character.
        self.must_read(b':')?;

        Ok(scheme)
    }

    fn exists_user_part_in_uri(&self) -> bool {
        self.remaining()
            .iter()
            .take_while(|&&b| !is_space(b) && !is_newline(b) && b != b'>')
            .any(|&b| b == b'@')
    }

    fn parse_user_info(&mut self) -> Result<Option<UserInfo>> {
        if !self.exists_user_part_in_uri() {
            return Ok(None);
        }

        // We have user part in uri.
        let user = self.read_user_str().into();
        let pass = if let Some(b':') = self.advance_if_eq(b':') {
            Some(self.read_pass_as_str().into())
        } else {
            None
        };

        // Take '@'.
        self.must_read(b'@')?;

        Ok(Some(UserInfo { user, pass }))
    }

    pub(crate) fn parse_host_port(&mut self) -> Result<HostPort> {
        let host = match self.peek_byte() {
            Some(b'[') => {
                // Is a Ipv6 host
                self.next_byte()?;
                // the '[' and ']' characters are removed from the host
                let host = self.read_while_as_str(|b| b != b']')?;
                self.next_byte()?;

                if let Ok(ipv6_addr) = host.parse() {
                    Host::IpAddr(ipv6_addr)
                } else {
                    return self.parse_error("Error parsing Ipv6 host!".into());
                }
            }
            _ => {
                // Is a domain name or Ipv4 host.
                let host = self.read_host_str();
                if host.is_empty() {
                    return self.parse_error("Can't parse the host!".into());
                }
                if let Ok(ip_addr) = host.parse() {
                    Host::IpAddr(ip_addr)
                } else {
                    Host::DomainName(DomainName(host.into()))
                }
            }
        };

        let port = self.parse_port()?;

        Ok(HostPort { host, port })
    }

    fn parse_port(&mut self) -> Result<Option<u16>> {
        let Some(b':') = self.advance_if_eq(b':') else {
            return Ok(None);
        };
        let port = self.scanner.read_u16()?;

        if crate::is_valid_port(port) {
            Ok(Some(port))
        } else {
            self.parse_error("Sip Uri Port is invalid!".into())
        }
    }

    pub(crate) fn parse_sip_addr(&mut self, parse_params: bool) -> Result<SipAddr> {
        self.space();

        match self.scanner.peek_bytes(3) {
            Some(SIP) | Some(SIPS) => {
                let uri = self.parse_uri(parse_params)?;
                Ok(SipAddr::Uri(uri))
            }
            _ => {
                let addr = self.parse_name_addr()?;
                Ok(SipAddr::NameAddr(addr))
            }
        }
    }

    pub(crate) fn parse_uri(&mut self, parse_params: bool) -> Result<Uri> {
        self.space();
        // "sip:" [ userinfo ] hostport uri-parameters [ headers ]
        let scheme = self.parse_scheme()?;
        let user = self.parse_user_info()?;
        let host_port = self.parse_host_port()?;

        if !parse_params {
            return Ok(Uri::new(scheme, user, host_port));
        }

        // Parse SIP uri parameters.
        let mut user_param = None;
        let mut method_param = None;
        let mut transport_param = None;
        let mut ttl_param = None;
        let mut lr_param: Option<&str> = None;
        let mut maddr_param = None;

        let parameters = parse_param!(
            self,
            parse_uri_param,
            USER_PARAM = user_param,
            METHOD_PARAM = method_param,
            TRANSPORT_PARAM = transport_param,
            TTL_PARAM = ttl_param,
            LR_PARAM = lr_param,
            MADDR_PARAM = maddr_param
        );

        let transport_param = transport_param.map(|s: &str| s.into());
        let ttl_param = ttl_param.map(|ttl: &str| ttl.parse().unwrap());
        let lr_param = lr_param.is_some();
        let method_param = method_param.map(|p: &str| p.as_bytes().into());
        let user_param = user_param.map(|u: &str| u.into());
        let maddr_param = maddr_param.and_then(|m: &str| m.parse::<Host>().ok());

        let headers = if let Some(b'?') = self.advance_if_eq(b'?') {
            // The uri has header parameters.
            Some(self.parse_headers_in_sip_uri()?)
        } else {
            None
        };
        self.space();

        Ok(Uri {
            scheme,
            user,
            host_port,
            transport_param,
            ttl_param,
            method_param,
            user_param,
            lr_param,
            maddr_param,
            parameters,
            headers,
        })
    }

    pub(crate) fn parse_name_addr(&mut self) -> Result<NameAddr> {
        self.space();
        let display = self.parse_display_name()?;
        self.space();

        // must be an '<'
        let Some(b'<') = self.scanner.next_byte() else {
            return self.parse_error("Expected '<' in NameAddr!".into());
        };

        let uri = self.parse_uri(true)?;

        // must be an '>'
        let Some(b'>') = self.scanner.next_byte() else {
            return self.parse_error("Expected '>' in NameAddr!".into());
        };

        Ok(NameAddr {
            display: display.map(|d| d.into()),
            uri,
        })
    }

    fn parse_headers_in_sip_uri(&mut self) -> Result<UriHeaders> {
        let mut params = Parameters::new();

        loop {
            let param = self.parse_hdr_in_uri()?;
            params.push(param);

            if self.advance_if_eq(b'&').is_none() {
                break;
            }
        }
        Ok(UriHeaders { inner: params })
    }

    fn parse_display_name(&mut self) -> Result<Option<&'buf str>> {
        match self.scanner.peek_byte() {
            Some(b'"') => {
                self.next_byte()?; // consume '"'
                let name = self.read_while(|b| b != b'"');
                self.next_byte()?; // consume closing '"'
                Ok(Some(str::from_utf8(name)?))
            }
            Some(b'<') => Ok(None), // no display name
            None => {
                return Err(crate::Error::Internal("EOF!"));
            }
            _ => {
                let name = self.parse_token()?;
                self.space();
                Ok(Some(name))
            }
        }
    }

    #[inline]
    pub(crate) fn parse_token(&mut self) -> Result<&'buf str> {
        if let Some(b'"') = self.advance_if_eq(b'"') {
            let value = self.read_while(|b| b != b'"');
            self.next_byte()?;

            Ok(str::from_utf8(value)?)
        } else {
            // is_token ensures that is valid UTF-8
            Ok(self.read_token_str())
        }
    }

    #[inline]
    pub(crate) fn next_byte(&mut self) -> Result<u8> {
        self.scanner
            .next_byte()
            .ok_or_else(|| self.parse_error::<u8>("EOF!".into()).unwrap_err())
    }

    /// Shortcut for yielding a parse error wrapped in a result type.
    pub(crate) fn parse_error<T>(&self, msg: String) -> Result<T> {
        parse_error!(msg, self.scanner)
    }

    /// Read until a new line (`\r` or `\n`) is found.
    pub(crate) fn read_until_new_line(&mut self) -> Result<&'buf str> {
        let bytes = self.read_while(is_not_newline);

        Ok(str::from_utf8(bytes)?)
    }

    fn read_while_as_str(&mut self, func: impl Fn(u8) -> bool) -> Result<&'buf str> {
        let bytes = self.read_while(func);

        Ok(str::from_utf8(bytes)?)
    }

    /// Read space characters.
    #[inline]
    pub(crate) fn space(&mut self) {
        self.read_while(is_space);
    }

    /// Advance to the next line.
    #[inline]
    pub(crate) fn new_line(&mut self) {
        self.read_while(is_newline);
    }

    /// Read alphabetic characters.
    #[inline]
    pub(crate) fn alphabetic(&mut self) -> &'buf [u8] {
        self.read_while(is_alphabetic)
    }

    #[inline]
    fn read_while(&mut self, func: impl Fn(u8) -> bool) -> &'buf [u8] {
        self.scanner.read_while(func)
    }

    #[inline]
    fn advance_if_eq(&mut self, byte: u8) -> Option<u8> {
        self.scanner.advance_if_eq(byte)
    }

    #[inline]
    fn must_read_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        Ok(self.scanner.must_read_bytes(bytes)?)
    }

    #[inline]
    pub(crate) fn read_until_byte(&mut self, byte: u8) -> &'buf [u8] {
        self.scanner.read_until(byte)
    }

    #[inline]
    pub(crate) fn peek_byte(&self) -> Option<&u8> {
        self.scanner.peek_byte()
    }

    #[inline]
    pub(crate) fn position(&self) -> &Position {
        self.scanner.position()
    }

    /// Get the remaining bytes in the scanner.
    #[inline]
    pub(crate) fn remaining(&self) -> &[u8] {
        self.scanner.remaining()
    }

    #[inline]
    pub(crate) fn not_comma_or_newline(&mut self) -> &'buf [u8] {
        self.read_while(not_comma_or_newline)
    }

    #[inline]
    pub(crate) fn is_next_newline(&self) -> bool {
        self.scanner.peek_byte().is_some_and(|&b| is_newline(b))
    }

    #[inline]
    pub(crate) fn parse_u32(&mut self) -> Result<u32> {
        Ok(self.scanner.read_u32()?)
    }

    #[inline]
    pub(crate) fn must_read(&mut self, byte: u8) -> Result<()> {
        Ok(self.scanner.must_read(byte)?)
    }

    #[inline]
    pub(crate) fn parse_f32(&mut self) -> Result<f32> {
        Ok(self.scanner.read_f32()?)
    }

    fn read_digits(&mut self) -> &'buf [u8] {
        self.read_while(is_digit)
    }

    #[inline]
    fn read_user_str(&mut self) -> &'buf str {
        unsafe { self.read_while_as_str_unchecked(is_user) }
    }

    #[inline]
    fn read_pass_as_str(&mut self) -> &'buf str {
        unsafe { self.read_while_as_str_unchecked(is_pass) }
    }

    #[inline]
    fn read_host_str(&mut self) -> &'buf str {
        unsafe { self.read_while_as_str_unchecked(is_host) }
    }

    #[inline]
    fn read_token_str(&mut self) -> &'buf str {
        unsafe { self.read_while_as_str_unchecked(is_token) }
    }

    #[inline]
    pub(crate) unsafe fn read_while_as_str_unchecked(
        &mut self,
        func: impl Fn(u8) -> bool,
    ) -> &'buf str {
        self.scanner.read_while_as_str_unchecked(func)
    }

    pub(crate) unsafe fn parse_param_unchecked(
        &mut self,
        func: impl Fn(u8) -> bool,
    ) -> Result<(&'buf str, Option<&'buf str>)> {
        self.space();

        let name = unsafe { self.scanner.read_while_as_str_unchecked(&func) };

        let Some(b'=') = self.scanner.peek_byte() else {
            return Ok((name, None));
        };

        self.next_byte()?;

        let value = if let Some(b'"') = self.scanner.peek_byte() {
            // TODO: skip ignore \"\"
            let Some(value) = self.scanner.read_between(b'"') else {
                return self.parse_error("Missing quoted end on parse param!".into());
            };

            str::from_utf8(value)?
        } else {
            unsafe { self.scanner.read_while_as_str_unchecked(func) }
        };

        Ok((name, Some(value)))
    }

    // Parse parameter (";" pname ["=" pvalue]).
    pub(crate) fn parse_param_ref(&mut self) -> Result<ParamRef<'buf>> {
        unsafe { self.parse_param_unchecked(is_token) }
    }

    pub(crate) fn parse_auth_credential(&mut self) -> Result<Credential> {
        let scheme = self.parse_token()?;

        if scheme == DIGEST {
            return self.parse_digest_credential();
        }

        self.parse_other_credential(scheme)
    }

    pub(crate) fn parse_auth_challenge(&mut self) -> Result<Challenge> {
        let scheme = self.parse_token()?;

        if scheme == DIGEST {
            return self.parse_digest_challenge();
        }

        let mut params = Parameters::new();

        comma_separated!(self => {
            let param = self.parse_param_ref()?.into();

            params.push(param);

        });

        Ok(Challenge::Other {
            scheme: scheme.into(),
            param: params,
        })
    }

    fn parse_digest_challenge(&mut self) -> Result<Challenge> {
        let mut digest = DigestChallenge::default();

        comma_separated!(self => {
            let Parameter {name, value} = self.parse_param_ref()?.into();

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

    fn parse_digest_credential(&mut self) -> Result<Credential> {
        let mut digest = DigestCredential::default();

        comma_separated!(self => {
            let Parameter { name, value } = self.parse_param_ref()?.into();
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

    fn parse_other_credential(&mut self, scheme: &'buf str) -> Result<Credential> {
        let mut param = Parameters::new();

        comma_separated!(self => {
            let mut p: Parameter = self.parse_param_ref()?.into();

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
    fn parse_hdr_in_uri(&mut self) -> Result<Parameter> {
        // SAFETY: `is_hdr_uri` only accepts ASCII bytes, which are
        // always valid UTF-8.
        Ok(unsafe { self.parse_param_unchecked(is_hdr_uri)?.into() })
    }
}

fn parse_uri_param<'a>(parser: &mut Parser<'a>) -> Result<ParamRef<'a>> {
    // SAFETY: `is_param` only accepts ASCII bytes, which are
    // always valid UTF-8.
    let mut param = unsafe { parser.parse_param_unchecked(is_param)? };

    if param.0 == LR_PARAM && param.1.is_none() {
        param.1 = Some("");
    }

    Ok(param)
}

#[inline]
pub(crate) fn parse_via_param<'a>(parser: &mut Parser<'a>) -> Result<ParamRef<'a>> {
    // SAFETY: `is_via_param` only accepts ASCII bytes, which
    // are always valid UTF-8.
    unsafe { parser.parse_param_unchecked(is_via_param) }
}

#[inline(always)]
fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t')
}

#[inline(always)]
fn is_newline(c: u8) -> bool {
    matches!(c, b'\r' | b'\n')
}

#[inline(always)]
fn is_not_newline(c: u8) -> bool {
    !is_newline(c)
}

#[inline(always)]
fn not_comma_or_newline(c: u8) -> bool {
    !is_newline(c) && c != b','
}

#[inline(always)]
fn is_alphabetic(c: u8) -> bool {
    c.is_ascii_alphabetic()
}

#[inline(always)]
fn is_digit(c: u8) -> bool {
    c.is_ascii_digit()
}

#[inline(always)]
pub(crate) fn is_via_param(b: u8) -> bool {
    VIA_PARAM_TAB[b as usize]
}

#[inline(always)]
pub(crate) fn is_host(b: u8) -> bool {
    HOST_TAB[b as usize]
}

#[inline(always)]
pub(crate) fn is_token(b: u8) -> bool {
    TOKEN_TAB[b as usize]
}

#[inline(always)]
fn is_user(b: u8) -> bool {
    USER_TAB[b as usize]
}

#[inline(always)]
fn is_pass(b: u8) -> bool {
    PASS_TAB[b as usize]
}

#[inline(always)]
fn is_param(b: u8) -> bool {
    PARAM_TAB[b as usize]
}

#[inline(always)]
fn is_hdr_uri(b: u8) -> bool {
    HDR_TAB[b as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::TransportType;
    use crate::{filter_map_header, find_map_header};

    macro_rules! uri_test_ok {
        (name: $name:ident, input: $input:literal, expected: $expected:expr) => {
            #[test]
            fn $name() -> Result<()> {
                let uri = Parser::new($input).parse_sip_addr(true)?;

                assert_eq!($expected.scheme, uri.scheme());
                assert_eq!($expected.host_port.host, uri.host_port().host);
                assert_eq!($expected.host_port.port, uri.host_port().port);
                assert_eq!($expected.user, uri.user().cloned());
                assert_eq!($expected.transport_param, uri.transport_param());
                assert_eq!(&$expected.ttl_param, uri.ttl_param());
                assert_eq!(&$expected.method_param, uri.method_param());
                assert_eq!(&$expected.user_param, uri.user_param());
                assert_eq!($expected.lr_param, uri.lr_param());
                assert_eq!(&$expected.maddr_param, uri.maddr_param());

                if let Some(params) = uri.other_params() {
                    assert!($expected.parameters.is_some(), "missing parameters!");
                    for param in $expected.parameters.unwrap().iter() {
                        assert_eq!(params.get_named(param.name()), param.value());
                    }
                }
                if let Some(headers) = uri.headers() {
                    assert!($expected.headers.is_some(), "missing headers!");
                    for param in $expected.headers.unwrap().iter() {
                        assert_eq!(headers.get_named(param.name()), param.value());
                    }
                }

                Ok(())
            }
        };
    }

    uri_test_ok! {
        name: uri_test_1,
        input: "sip:biloxi.com",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .host("biloxi.com".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_2,
        input: "sip:biloxi.com:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .host("biloxi.com:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_3,
        input: "sip:a@b:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("a", None))
            .host("b:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_4,
        input: "sip:bob@biloxi.com:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_5,
        input: "sip:bob@192.0.2.201:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("192.0.2.201:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_6,
        input: "sip:bob@[::1]:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("[::1]:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_7,
        input: "sip:bob:secret@biloxi.com",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", Some("secret")))
            .host("biloxi.com".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_8,
        input: "sip:bob:pass@192.0.2.201",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", Some("pass")))
            .host("192.0.2.201".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_9,
        input: "sip:bob@biloxi.com;foo=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com".parse().unwrap())
            .param("foo", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: uri_test_10,
        input: "sip:bob@biloxi.com:5060;foo=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .param("foo", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: uri_test_11,
        input: "sips:bob@biloxi.com:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sips)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: uri_test_12,
        input: "sips:bob:pass@biloxi.com:5060",
        expected: Uri::builder()
            .scheme(Scheme::Sips)
            .user(UserInfo::new("bob", Some("pass")))
            .host("biloxi.com:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: test_uri_11,
        input: "sip:bob@biloxi.com:5060;foo",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .param("foo", None)
            .host("biloxi.com:5060".parse().unwrap())
            .build()
    }

    uri_test_ok! {
        name: test_uri_12,
        input: "sip:bob@biloxi.com:5060;foo;baz=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .param("baz", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_13,
        input: "sip:bob@biloxi.com:5060;baz=bar;foo",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .param("baz", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_14,
        input: "sip:bob@biloxi.com:5060;baz=bar;foo;a=b",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .param("baz", Some("bar"))
            .param("foo", None)
            .param("a", Some("b"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_15,
        input: "sip:bob@biloxi.com?foo=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com".parse().unwrap())
            .header("foo", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_16,
        input: "sip:bob@biloxi.com?foo",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com".parse().unwrap())
            .header("foo", None)
            .build()
    }

    uri_test_ok! {
        name: test_uri_17,
        input: "sip:bob@biloxi.com:5060?foo=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .header("foo", Some("bar"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_18,
        input: "sip:bob@biloxi.com:5060?baz=bar&foo=&a=b",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .header("baz", Some("bar"))
            .header("foo", Some(""))
            .header("a", Some("b"))
            .build()
    }

    uri_test_ok! {
        name: test_uri_19,
        input: "sip:bob@biloxi.com:5060?foo=bar&baz",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com:5060".parse().unwrap())
            .header("foo", Some("bar"))
            .header("baz", None)
            .build()
    }

    uri_test_ok! {
        name: test_uri_20,
        input: "sip:bob@biloxi.com;foo?foo=bar",
        expected: Uri::builder()
            .scheme(Scheme::Sip)
            .user(UserInfo::new("bob", None))
            .host("biloxi.com".parse().unwrap())
            .param("foo", None)
            .header("foo", Some("bar"))
            .build()
    }

    #[test]
    fn test_parse_request() {
        let buf = concat! {
            "INVITE sip:bob@biloxi.example.com SIP/2.0\r\n",
            "Via: SIP/2.0/TCP client.atlanta.example.com:5060;branch=z9hG4bK74b43\r\n",
            "Max-Forwards: 70\r\n",
            "Route: <sip:ss1.atlanta.example.com;lr>\r\n",
            "From: Alice <sip:alice@atlanta.example.com>;tag=9fxced76sl\r\n",
            "To: Bob <sip:bob@biloxi.example.com>\r\n",
            "Call-ID: 3848276298220188511@atlanta.example.com\r\n",
            "CSeq: 1 INVITE\r\n",
            "Contact: <sip:alice@client.atlanta.example.com;transport=tcp>\r\n",
            "Content-Type: application/sdp\r\n",
            "Content-Length: 151\r\n",
            "\r\n",
            "v=0\r\n",
            "o=alice 2890844526 2890844526 IN IP4 client.atlanta.example.com\r\n",
            "s=-\r\n",
            "c=IN IP4 192.0.2.101\r\n",
            "t=0 0\r\n",
            "m=audio 49172 RTP/AVP 0\r\n",
            "a=rtpmap:0 PCMU/8000\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let req = msg.request().unwrap();

        assert_eq!(req.req_line.method, SipMethod::Invite);
        assert_eq!(req.req_line.uri.to_string(), "sip:bob@biloxi.example.com");

        let via = find_map_header!(req.headers, Via).unwrap();
        assert_eq!(via.transport(), TransportType::Tcp);
        assert_eq!(via.sent_by().to_string(), "client.atlanta.example.com:5060");
        assert_eq!(via.branch().unwrap(), "z9hG4bK74b43");

        let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
        assert_eq!(maxfowards.max_fowards(), 70);

        let route = find_map_header!(req.headers, Route).unwrap();
        assert_eq!(route.addr.uri.to_string(), "sip:ss1.atlanta.example.com;lr");

        let from = find_map_header!(req.headers, From).unwrap();
        assert_eq!(from.display(), Some("Alice"));
        assert_eq!(from.tag().as_deref(), Some("9fxced76sl"));

        let to = find_map_header!(req.headers, To).unwrap();
        assert_eq!(to.display(), Some("Bob"));
        assert_eq!(to.uri().to_string(), "sip:bob@biloxi.example.com");

        let call_id = find_map_header!(req.headers, CallId).unwrap();
        assert_eq!(call_id.id(), "3848276298220188511@atlanta.example.com");

        let cseq = find_map_header!(req.headers, CSeq).unwrap();
        assert_eq!(cseq.cseq, 1);
        assert_eq!(cseq.method, SipMethod::Invite);

        let contact = find_map_header!(req.headers, Contact).unwrap();
        let host_str = contact.uri.uri().host_port.host_as_str();
        assert_eq!(host_str, "client.atlanta.example.com");

        let content_type = find_map_header!(req.headers, ContentType).unwrap();
        assert_eq!(content_type.media_type().to_string(), "application/sdp");

        let content_length = find_map_header!(req.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 151);

        assert_eq!(
            req.body.as_deref().unwrap(),
            concat!(
                "v=0\r\n",
                "o=alice 2890844526 2890844526 IN IP4 client.atlanta.example.com\r\n",
                "s=-\r\n",
                "c=IN IP4 192.0.2.101\r\n",
                "t=0 0\r\n",
                "m=audio 49172 RTP/AVP 0\r\n",
                "a=rtpmap:0 PCMU/8000\r\n"
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_parse_request_without_body() {
        let buf = concat! {
            "INVITE sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@example.com>\r\n",
            "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Contact: <sip:alice@example.com>\r\n",
            "Content-Length: 0\r\n",
            "\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let req = msg.request().unwrap();

        assert_eq!(req.req_line.method, SipMethod::Invite);
        assert_eq!(req.req_line.uri.to_string(), "sip:bob@example.com");

        let via = find_map_header!(req.headers, Via).unwrap();
        assert_eq!(via.transport(), TransportType::Udp);
        assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");
        assert_eq!(via.branch().unwrap(), "z9hG4bK776asdhds");

        let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
        assert_eq!(maxfowards.max_fowards(), 70);

        let to = find_map_header!(req.headers, To).unwrap();
        assert_eq!(to.uri().to_string(), "sip:bob@example.com");
        assert_eq!(to.display(), Some("Bob"));

        let from = find_map_header!(req.headers, From).unwrap();
        assert_eq!(from.display(), Some("Alice"));
        assert_eq!(from.uri().to_string(), "sip:alice@example.com");

        let call_id = find_map_header!(req.headers, CallId).unwrap();
        assert_eq!(call_id.id(), "a84b4c76e66710");

        let cseq = find_map_header!(req.headers, CSeq).unwrap();
        assert_eq!(cseq.cseq, 314159);

        let contact = find_map_header!(req.headers, Contact).unwrap();
        assert_eq!(contact.uri.to_string(), "<sip:alice@example.com>");

        let content_length = find_map_header!(req.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 0);
    }

    #[test]
    fn test_parse_response() {
        let buf = concat! {
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "From: Alice <sip:alice@atlanta.com>;tag=1928301774\r\n",
            "To: Bob <sip:bob@example.com>;tag=a6c85cf\r\n",
            "Call-ID: a84b4c76e66710@pc33.atlanta.com\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Contact: <sip:bob@biloxi.com>\r\n",
            "Content-Type: application/sdp\r\n",
            "Content-Length: 131\r\n",
            "\r\n",
            "v=0\r\n",
            "o=bob 2808844564 2808844564 IN IP4 biloxi.com\r\n",
            "s=-\r\n",
            "c=IN IP4 biloxi.com\r\n",
            "t=0 0\r\n",
            "m=audio 7078 RTP/AVP 0\r\n",
            "a=rtpmap:0 PCMU/8000\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let resp = msg.response().unwrap();

        assert_eq!(resp.code().as_u16(), 200);
        assert_eq!(resp.reason(), "OK");

        let via = find_map_header!(resp.headers, Via).unwrap();
        assert_eq!(via.transport(), TransportType::Udp);
        assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");

        let content_type = find_map_header!(resp.headers, ContentType).unwrap();
        assert_eq!(content_type.media_type().to_string(), "application/sdp");

        let content_length = find_map_header!(resp.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 131);

        assert_eq!(
            resp.body.as_deref().unwrap(),
            concat!(
                "v=0\r\n",
                "o=bob 2808844564 2808844564 IN IP4 biloxi.com\r\n",
                "s=-\r\n",
                "c=IN IP4 biloxi.com\r\n",
                "t=0 0\r\n",
                "m=audio 7078 RTP/AVP 0\r\n",
                "a=rtpmap:0 PCMU/8000\r\n"
            )
            .as_bytes()
        );
    }

    #[test]
    fn test_parse_response_without_body() {
        let buf = concat! {
            "SIP/2.0 200 OK\r\n",
            "Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bK776asdhds\r\n",
            "Max-Forwards: 70\r\n",
            "To: Bob <sip:bob@example.com>\r\n",
            "From: Alice <sip:alice@example.com>;tag=1928301774\r\n",
            "Call-ID: a84b4c76e66710\r\n",
            "CSeq: 314159 INVITE\r\n",
            "Content-Length: 0\r\n\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let resp = msg.response().unwrap();

        assert_eq!(resp.code().as_u16(), 200);
        assert_eq!(resp.reason(), "OK");

        let via = find_map_header!(resp.headers, Via).unwrap();
        assert_eq!(via.transport(), TransportType::Udp);
        assert_eq!(via.sent_by().to_string(), "pc33.atlanta.com");

        let maxfowards = find_map_header!(resp.headers, MaxForwards).unwrap();
        assert_eq!(maxfowards.max_fowards(), 70);

        let to = find_map_header!(resp.headers, To).unwrap();
        assert_eq!(to.uri().to_string(), "sip:bob@example.com");
        assert_eq!(to.display(), Some("Bob"));

        let from = find_map_header!(resp.headers, From).unwrap();
        assert_eq!(from.display(), Some("Alice"));
        assert_eq!(from.uri().to_string(), "sip:alice@example.com");

        let call_id = find_map_header!(resp.headers, CallId).unwrap();
        assert_eq!(call_id.id(), "a84b4c76e66710");

        let cseq = find_map_header!(resp.headers, CSeq).unwrap();
        assert_eq!(cseq.cseq, 314159);
        assert_eq!(cseq.method, SipMethod::Invite);

        let content_length = find_map_header!(resp.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 0);
    }

    #[test]
    fn test_parse_request_with_multiple_via_headers() {
        let buf = concat! {
            "REGISTER sip:registrar.example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP host1.example.com;branch=z9hG4bK111\r\n",
            "Via: SIP/2.0/UDP host2.example.com;branch=z9hG4bK222\r\n",
            "Via: SIP/2.0/UDP host3.example.com;branch=z9hG4bK333\r\n",
            "Max-Forwards: 70\r\n",
            "To: <sip:alice@example.com>\r\n",
            "From: <sip:alice@example.com>;tag=1928301774\r\n",
            "Call-ID: manyvias@atlanta.com\r\n",
            "CSeq: 42 REGISTER\r\n",
            "Contact: <sip:alice@pc33.atlanta.com>\r\n",
            "Content-Length: 0\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let req = msg.request().unwrap();

        assert_eq!(req.req_line.method, SipMethod::Register);
        assert_eq!(req.req_line.uri.to_string(), "sip:registrar.example.com");

        let vias: Vec<_> = filter_map_header!(req.headers, Via).collect();
        assert_eq!(vias.len(), 3);
        assert_eq!(vias[0].sent_by().to_string(), "host1.example.com");
        assert_eq!(vias[0].branch().unwrap(), "z9hG4bK111");
        assert_eq!(vias[1].sent_by().to_string(), "host2.example.com");
        assert_eq!(vias[1].branch().unwrap(), "z9hG4bK222");
        assert_eq!(vias[2].sent_by().to_string(), "host3.example.com");
        assert_eq!(vias[2].branch().unwrap(), "z9hG4bK333");

        let max_forwards = find_map_header!(req.headers, MaxForwards).unwrap();
        assert_eq!(max_forwards.max_fowards(), 70);

        let to = find_map_header!(req.headers, To).unwrap();
        assert_eq!(to.uri().to_string(), "sip:alice@example.com");

        let from = find_map_header!(req.headers, From).unwrap();
        assert_eq!(from.uri().to_string(), "sip:alice@example.com");
        assert_eq!(from.tag().as_deref(), Some("1928301774"));

        let call_id = find_map_header!(req.headers, CallId).unwrap();
        assert_eq!(call_id.id(), "manyvias@atlanta.com");

        let cseq = find_map_header!(req.headers, CSeq).unwrap();
        assert_eq!(cseq.cseq, 42);
        assert_eq!(cseq.method, SipMethod::Register);

        let contact = find_map_header!(req.headers, Contact).unwrap();
        assert_eq!(contact.uri.to_string(), "<sip:alice@pc33.atlanta.com>");

        let content_length = find_map_header!(req.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 0);

        assert!(req.body.is_none());
    }

    #[test]
    fn test_header_with_multi_params() {
        let buf = concat! {
            "OPTIONS sip:bob@example.com SIP/2.0\r\n",
            "Via: SIP/2.0/UDP folded.example.com;branch=z9hG4bKfolded\r\n",
            "Max-Forwards: 70\r\n",
            "To: <sip:bob@example.com>\r\n",
            "From: <sip:alice@atlanta.com>;tag=1928301774\r\n",
            "Call-ID: foldedoptions@atlanta.com\r\n",
            "CSeq: 100 OPTIONS\r\n",
            "Contact: <sip:alice@atlanta.com>;",
            " param1=value1;",
            " param2=value2;",
            " param3=value3;",
            " param4=value4\r\n",
            "Content-Length: 0\r\n\r\n"
        };

        let msg = Parser::parse_sip_msg(buf).unwrap();
        let req = msg.request().unwrap();

        assert_eq!(req.req_line.method, SipMethod::Options);
        assert_eq!(req.req_line.uri.to_string(), "sip:bob@example.com");

        let via = find_map_header!(req.headers, Via).unwrap();
        assert_eq!(via.transport(), TransportType::Udp);
        assert_eq!(via.sent_by().to_string(), "folded.example.com");

        let maxfowards = find_map_header!(req.headers, MaxForwards).unwrap();
        assert_eq!(maxfowards.max_fowards(), 70);

        let to = find_map_header!(req.headers, To).unwrap();
        assert_eq!(to.uri().to_string(), "sip:bob@example.com");

        let from = find_map_header!(req.headers, From).unwrap();
        assert_eq!(from.uri().to_string(), "sip:alice@atlanta.com");

        let call_id = find_map_header!(req.headers, CallId).unwrap();
        assert_eq!(call_id.id(), "foldedoptions@atlanta.com");

        let cseq = find_map_header!(req.headers, CSeq).unwrap();
        assert_eq!(cseq.cseq, 100);
        assert_eq!(cseq.method, SipMethod::Options);

        let contact = find_map_header!(req.headers, Contact).unwrap();
        let params = contact.param.as_ref().unwrap();
        assert_eq!(contact.uri.to_string(), "<sip:alice@atlanta.com>");
        assert_eq!(params.get_named("param1"), Some("value1"));
        assert_eq!(params.get_named("param2"), Some("value2"));
        assert_eq!(params.get_named("param3"), Some("value3"));
        assert_eq!(params.get_named("param4"), Some("value4"));

        let content_length = find_map_header!(req.headers, ContentLength).unwrap();
        assert_eq!(content_length.clen(), 0);
    }
}
