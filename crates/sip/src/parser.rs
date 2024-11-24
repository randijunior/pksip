//! SIP Parser

use std::str::Utf8Error;

use reader::newline;
use reader::peek_while;
use reader::space;
use reader::util::is_alphabetic;
use reader::Reader;

use crate::headers::Accept;
use crate::headers::AcceptEncoding;
use crate::headers::AcceptLanguage;
use crate::headers::AlertInfo;
use crate::headers::Allow;
use crate::headers::AuthenticationInfo;
use crate::headers::Authorization;
use crate::headers::CSeq;
use crate::headers::CallId;
use crate::headers::Contact;
use crate::headers::ContentDisposition;
use crate::headers::ContentEncoding;
use crate::headers::ContentLength;
use crate::headers::ContentType;
use crate::headers::Date;
use crate::headers::ErrorInfo;
use crate::headers::Expires;
use crate::headers::Header;
use crate::headers::InReplyTo;
use crate::headers::MaxForwards;
use crate::headers::MimeVersion;
use crate::headers::MinExpires;
use crate::headers::Organization;
use crate::headers::Priority;
use crate::headers::ProxyAuthenticate;
use crate::headers::ProxyAuthorization;
use crate::headers::ProxyRequire;
use crate::headers::RecordRoute;
use crate::headers::ReplyTo;
use crate::headers::Require;
use crate::headers::RetryAfter;
use crate::headers::Route;
use crate::headers::Server;
use crate::headers::SipHeader;
use crate::headers::Subject;
use crate::headers::Supported;
use crate::headers::Timestamp;
use crate::headers::To;
use crate::headers::Unsupported;
use crate::headers::UserAgent;
use crate::headers::Via;
use crate::headers::WWWAuthenticate;
use crate::headers::Warning;
use crate::macros::sip_parse_error;
use crate::message::Token;
use crate::message::SIP;

/// Result for sip parser
pub type Result<T> = std::result::Result<T, SipParserError>;

use crate::headers::Headers;

use crate::message::SipMessage;
use crate::message::StatusLine;
use crate::message::{RequestLine, SipRequest, SipResponse};

pub(crate) const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

pub(crate) const ALPHA_NUM: &[u8] = b"abcdefghijklmnopqrstuvwxyz\
                                    ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                    0123456789";

pub(crate) const UNRESERVED: &[u8] = b"-_.!~*'()%";
pub(crate) const ESCAPED: &[u8] = b"%";
pub(crate) const USER_UNRESERVED: &[u8] = b"&=+$,;?/";
pub(crate) const TOKEN: &[u8] = b"-.!%*_`'~+";
pub(crate) const PASS: &[u8] = b"&=+$,";
pub(crate) const HOST: &[u8] = b"_-.";
pub(crate) const GENERIC_URI: &[u8] = b"#?;:@&=+-_.!~*'()%$,/";

pub struct SipParser;

impl SipParser {
    /// Parse a buff of bytes into sip message
    pub fn parse<'a>(buff: &'a [u8]) -> Result<SipMessage<'a>> {
        let mut reader = Reader::new(buff);

        let mut msg = if Self::is_sip_version(&reader) {
            SipMessage::Response(SipResponse {
                st_line: StatusLine::parse(&mut reader)?,
                headers: Headers::new(),
                body: None,
            })
        } else {
            SipMessage::Request(SipRequest {
                req_line: RequestLine::parse(&mut reader)?,
                headers: Headers::new(),
                body: None,
            })
        };

        let mut has_content_type = false;
        let reader = &mut reader;

        'headers: loop {
            let name = Token::parse(reader)?;

            if reader.next() != Some(&b':') {
                return sip_parse_error!("Invalid sip Header!");
            }
            space!(reader);

            match name {
                error_info if ErrorInfo::match_name(error_info) => {
                    let error_info = ErrorInfo::parse(reader)?;
                    msg.push_header(Header::ErrorInfo(error_info))
                }

                route if Route::match_name(route) => 'route: loop {
                    let route = Route::parse(reader)?;
                    msg.push_header(Header::Route(route));
                    let Some(&b',') = reader.peek() else {
                        break 'route;
                    };
                    reader.next();
                },

                via if Via::match_name(via) => 'via: loop {
                    let via = Via::parse(reader)?;
                    msg.push_header(Header::Via(via));
                    let Some(&b',') = reader.peek() else {
                        break 'via;
                    };
                    reader.next();
                },

                max_fowards if MaxForwards::match_name(max_fowards) => {
                    let max_fowards = MaxForwards::parse(reader)?;
                    msg.push_header(Header::MaxForwards(max_fowards))
                }

                from if crate::headers::From::match_name(from) => {
                    let from = crate::headers::From::parse(reader)?;
                    msg.push_header(Header::From(from))
                }

                to if To::match_name(to) => {
                    let to = To::parse(reader)?;
                    msg.push_header(Header::To(to))
                }

                cid if CallId::match_name(cid) => {
                    let call_id = CallId::parse(reader)?;
                    msg.push_header(Header::CallId(call_id))
                }

                cseq if CSeq::match_name(cseq) => {
                    let cseq = CSeq::parse(reader)?;
                    msg.push_header(Header::CSeq(cseq))
                }

                auth if Authorization::match_name(auth) => {
                    let auth = Authorization::parse(reader)?;
                    msg.push_header(Header::Authorization(auth))
                }

                contact if Contact::match_name(contact) => 'contact: loop {
                    let contact = Contact::parse(reader)?;
                    msg.push_header(Header::Contact(contact));
                    let Some(&b',') = reader.peek() else {
                        break 'contact;
                    };
                    reader.next();
                },

                expires if Expires::match_name(expires) => {
                    let expires = Expires::parse(reader)?;
                    msg.push_header(Header::Expires(expires));
                }

                in_reply_to if InReplyTo::match_name(in_reply_to) => {
                    let in_reply_to = InReplyTo::parse(reader)?;
                    msg.push_header(Header::InReplyTo(in_reply_to));
                }

                mime_version if MimeVersion::match_name(mime_version) => {
                    let mime_version = MimeVersion::parse(reader)?;
                    msg.push_header(Header::MimeVersion(mime_version));
                }

                min_expires if MinExpires::match_name(min_expires) => {
                    let min_expires = MinExpires::parse(reader)?;
                    msg.push_header(Header::MinExpires(min_expires));
                }

                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(reader)?;
                    msg.push_header(Header::UserAgent(user_agent))
                }

                date if Date::match_name(date) => {
                    let date = Date::parse(reader)?;
                    msg.push_header(Header::Date(date))
                }

                server if Server::match_name(server) => {
                    let server = Server::parse(reader)?;
                    msg.push_header(Header::Server(server))
                }

                subject if Subject::match_name(subject) => {
                    let subject = Subject::parse(reader)?;
                    msg.push_header(Header::Subject(subject))
                }

                priority if Priority::match_name(priority) => {
                    let priority = Priority::parse(reader)?;
                    msg.push_header(Header::Priority(priority))
                }

                proxy_authenticate
                    if ProxyAuthenticate::match_name(proxy_authenticate) =>
                {
                    let proxy_authenticate = ProxyAuthenticate::parse(reader)?;
                    msg.push_header(Header::ProxyAuthenticate(
                        proxy_authenticate,
                    ))
                }

                proxy_authorization
                    if ProxyAuthorization::match_name(proxy_authorization) =>
                {
                    let proxy_authorization =
                        ProxyAuthorization::parse(reader)?;
                    msg.push_header(Header::ProxyAuthorization(
                        proxy_authorization,
                    ))
                }

                proxy_require if ProxyRequire::match_name(proxy_require) => {
                    let proxy_require = ProxyRequire::parse(reader)?;
                    msg.push_header(Header::ProxyRequire(proxy_require))
                }

                reply_to if ReplyTo::match_name(reply_to) => {
                    let reply_to = ReplyTo::parse(reader)?;
                    msg.push_header(Header::ReplyTo(reply_to))
                }

                content_length if ContentLength::match_name(content_length) => {
                    let content_length = ContentLength::parse(reader)?;
                    msg.push_header(Header::ContentLength(content_length))
                }

                content_encoding
                    if ContentEncoding::match_name(content_encoding) =>
                {
                    let content_encoding = ContentEncoding::parse(reader)?;
                    msg.push_header(Header::ContentEncoding(content_encoding))
                }

                content_type if ContentType::match_name(content_type) => {
                    let content_type = ContentType::parse(reader)?;
                    has_content_type = true;
                    msg.push_header(Header::ContentType(content_type))
                }

                content_disposition
                    if ContentDisposition::match_name(content_disposition) =>
                {
                    let content_disposition =
                        ContentDisposition::parse(reader)?;
                    msg.push_header(Header::ContentDisposition(
                        content_disposition,
                    ))
                }

                record_route if RecordRoute::match_name(record_route) => {
                    'rr: loop {
                        let record_route = RecordRoute::parse(reader)?;
                        msg.push_header(Header::RecordRoute(record_route));
                        let Some(&b',') = reader.peek() else {
                            break 'rr;
                        };
                        reader.next();
                    }
                }

                require if Require::match_name(require) => {
                    let require = Require::parse(reader)?;
                    msg.push_header(Header::Require(require))
                }

                retry_after if RetryAfter::match_name(retry_after) => {
                    let retry_after = RetryAfter::parse(reader)?;
                    msg.push_header(Header::RetryAfter(retry_after))
                }

                organization if Organization::match_name(organization) => {
                    let organization = Organization::parse(reader)?;
                    msg.push_header(Header::Organization(organization))
                }

                accept_encoding
                    if AcceptEncoding::match_name(accept_encoding) =>
                {
                    let accept_encoding = AcceptEncoding::parse(reader)?;
                    msg.push_header(Header::AcceptEncoding(accept_encoding));
                }

                accept if Accept::match_name(accept) => {
                    let accept = Accept::parse(reader)?;
                    msg.push_header(Header::Accept(accept));
                }

                accept_language
                    if AcceptLanguage::match_name(accept_language) =>
                {
                    let accept_language = AcceptLanguage::parse(reader)?;
                    msg.push_header(Header::AcceptLanguage(accept_language));
                }

                alert_info if AlertInfo::match_name(alert_info) => {
                    let alert_info = AlertInfo::parse(reader)?;
                    msg.push_header(Header::AlertInfo(alert_info));
                }

                allow if Allow::match_name(allow) => {
                    let allow = Allow::parse(reader)?;
                    msg.push_header(Header::Allow(allow));
                }

                auth_info if AuthenticationInfo::match_name(auth_info) => {
                    let auth_info = AuthenticationInfo::parse(reader)?;
                    msg.push_header(Header::AuthenticationInfo(auth_info));
                }

                supported if Supported::match_name(supported) => {
                    let supported = Supported::parse(reader)?;
                    msg.push_header(Header::Supported(supported));
                }

                timestamp if Timestamp::match_name(timestamp) => {
                    let timestamp = Timestamp::parse(reader)?;
                    msg.push_header(Header::Timestamp(timestamp));
                }

                user_agent if UserAgent::match_name(user_agent) => {
                    let user_agent = UserAgent::parse(reader)?;
                    msg.push_header(Header::UserAgent(user_agent));
                }

                unsupported if Unsupported::match_name(unsupported) => {
                    let unsupported = Unsupported::parse(reader)?;
                    msg.push_header(Header::Unsupported(unsupported));
                }

                www_authenticate
                    if WWWAuthenticate::match_name(www_authenticate) =>
                {
                    let www_authenticate = WWWAuthenticate::parse(reader)?;
                    msg.push_header(Header::WWWAuthenticate(www_authenticate));
                }

                warning if Warning::match_name(warning) => {
                    let warning = Warning::parse(reader)?;
                    msg.push_header(Header::Warning(warning));
                }

                _ => {
                    let value = Token::parse(reader)?;

                    msg.push_header(Header::Other { name, value });
                }
            };

            newline!(reader);
            if reader.is_eof() {
                break 'headers;
            }
        }

        if has_content_type {
            msg.set_body(Some(&buff[reader.idx()..]));
        }

        Ok(msg)
    }

    fn is_sip_version(reader: &Reader) -> bool {
        let tag = reader.peek_n(4);

        tag.is_some_and(|next| tag == Some(SIP) && &next[3] == &b'/')
    }

    pub fn parse_sip_v2(reader: &mut Reader) -> Result<()> {
        if let Some(SIPV2) = reader.peek_n(7) {
            reader.nth(6);
            return Ok(());
        }
        sip_parse_error!("Sip Version Invalid")
    }
}

/// Error on parsing
#[derive(Debug)]
pub struct SipParserError {
    /// Message in error
    pub message: String,
}

#[allow(missing_docs)]
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

impl<'a> From<reader::Error<'a>> for SipParserError {
    fn from(err: reader::Error) -> Self {
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
    use crate::{
        message::{Scheme, SipUri},
        message::{SipMethod, Transport},
    };

    use super::*;

    #[test]
    fn test_msg_1() {
        assert_matches!(SipParser::parse(
            b"INVITE sip:bob@biloxi.com SIP/2.0\r\n\
        Via: SIP/2.0/UDP pc33.atlanta.com;branch=z9hG4bKkjshdyff\r\n\
        To: Bob <sip:bob@biloxi.com>\r\n\
        From: Alice <sip:alice@atlanta.com>;tag=88sja8x\r\n\
        Max-Forwards: 70\r\n\
        Call-ID: 987asjd97y7atg\r\n\
        CSeq: 986759 INVITE\r\n",
        )
        .unwrap(), SipMessage::Request(req) => {
            assert_eq!(req.req_line.method, SipMethod::Invite);
            assert_eq!(req.req_line.uri.scheme, Scheme::Sip);
            assert_eq!(req.req_line.uri.user.unwrap().user, "bob");
            assert_eq!(req.req_line.uri.host.host_as_string(), String::from("biloxi.com"));

            assert!(req.headers.len() == 6);
            let mut iter = req.headers.iter();
            assert_matches!(iter.next().unwrap(), Header::Via(via) => {
                assert_eq!(via.transport, Transport::UDP);
                assert_eq!(via.sent_by.host_as_string(), "pc33.atlanta.com");
                assert_eq!(via.params.as_ref().unwrap().branch(), Some("z9hG4bKkjshdyff"));
            });

            assert_matches!(iter.next().unwrap(), Header::To(to) => {
                assert_matches!(&to.uri, SipUri::NameAddr(addr) => {
                    assert_eq!(addr.display, Some("Bob"));
                    assert_eq!(addr.uri.scheme, Scheme::Sip);
                    assert_eq!(addr.uri.user.as_ref().unwrap().user, "bob");
                });
            });
        })
    }
}
