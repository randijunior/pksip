use crate::{
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    reader::{InputReader, ReaderError},
    uri::{Host, Scheme, Uri, UserInfo},
    util::{is_alphabetic, is_digit, is_newline, is_space},
};

use std::{
    net::{IpAddr, Ipv6Addr},
    str::{self, FromStr},
};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl From<ReaderError> for SipParserError {
    fn from(err: ReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line: {} column: {}, kind: {:?}",
                err.line(),
                err.col(),
                err.kind()
            ),
        }
    }
}

pub struct SipParser<'parser> {
    reader: InputReader<'parser>,
}

impl<'parser> SipParser<'parser> {
    pub fn new(i: &'parser [u8]) -> Self {
        SipParser {
            reader: InputReader::new(i),
        }
    }
    pub fn parse_sip_version(&mut self) -> Result<(), SipParserError> {
        self.reader.tag(SIPV2)?;
        Ok(())
    }

    pub fn parse_status_line(&mut self) -> Result<StatusLine, SipParserError> {
        self.parse_sip_version()?;

        self.reader.read_while(is_space)?;

        let status_code = self.reader.read_while(is_digit)?;
        let status_code = SipStatusCode::from(status_code);

        self.reader.read_while(is_space)?;

        let rp = self.reader.read_while(|c| c != b'\r' && c != b'\n')?;

        let rp = str::from_utf8(rp).map_err(|_| SipParserError {
            message: "Reason phrase is invalid utf8!".to_string(),
        })?;

        Ok(StatusLine::new(status_code, rp))
    }

    fn parse_scheme(&mut self) -> Result<Scheme, SipParserError> {
        match self.reader.read_until(|b| b == b':')? {
            b"sip" => Ok(Scheme::Sip),
            b"sips" => Ok(Scheme::Sips),
            _ => Err(SipParserError {
                message: "Can't parse sip uri scheme".to_string(),
            }),
        }
    }

    fn parse_uri_host(&mut self) -> Result<Host, SipParserError> {
        if let Some(_) = self.reader.read_next_if(|b| b == b'[')? {
            let host = self.reader.read_until_byte(b']')?;

            let host = str::from_utf8(host).map_err(|_| SipParserError {
                message: "Sip host is invalid utf8!".to_string(),
            })?;
            let host: Ipv6Addr = host.parse().expect("Error parsing host addr!");

            self.reader.read()?;

            Ok(Host::IpAddr(std::net::IpAddr::V6(host)))
        } else {
            let host = self
                .reader
                .read_until(|b| b == b';' || b == b':' || b == b'?' || b == b' ' || is_newline(b))?;

            let host = str::from_utf8(host).map_err(|_| SipParserError {
                message: "Sip host is invalid utf8!".to_string(),
            })?;

            if let Ok(addr) = IpAddr::from_str(host) {
                Ok(Host::IpAddr(addr))
            } else {
                Ok(Host::DomainName(host.to_string()))
            }
        }
        
    }

    fn parse_port(&mut self) -> Result<Option<u32>, SipParserError> {
        if let Some(_) = self.reader.read_next_if(|b| b == b':')? {
            let digits = self.reader.read_while(is_digit)?;
            let digits = std::str::from_utf8(digits)
                .map_err(|_| "Invalid UTF-8")
                .and_then(|s| s.parse::<u32>().map_err(|_| "Parse error"))
                .unwrap();
            Ok(Some(digits))
        } else {
            Ok(None)
        }
    }

    fn parse_user(&mut self) -> Result<Option<UserInfo>, SipParserError> {
        if let Some(b'@') = self.reader.peek_for_match(b"@ \n>") {
            let user_part = self.reader.read_until_byte(b'@')?;

            if user_part.contains(&b':') {
                let mut parts = user_part.split(|&c| c == b':');
                let user = parts.next().unwrap();
                let pass = parts.next().unwrap();

                let user = str::from_utf8(user).unwrap();
                let pass = str::from_utf8(pass).unwrap();

                Ok(Some(UserInfo::new(user.to_string(), Some(pass.to_string()))))
            } else {
                let user = str::from_utf8(user_part).unwrap();
                Ok(Some(UserInfo::new(user.to_string(), None)))
            }
        } else {
            Ok(None)
        }
        
    }

    fn parse_method(&mut self) -> Result<SipMethod, SipParserError> {
        let method_bytes = self.reader.read_while(is_alphabetic)?;
        let method = SipMethod::from(method_bytes);
        self.reader.read_while(is_space)?;
        
        Ok(method)
    }

    // INVITE sip:alice@atlanta.com
    // sip:alice:secretword@atlanta.com;transport=tcp
    // sip:alice@192.0.2.4
    // sip:localhost
    // sip:alice;day=tuesday@atlanta.com
    pub fn parse_request_line(&mut self) -> Result<RequestLine, SipParserError> {
        let method = self.parse_method()?;

        let scheme = self.parse_scheme()?;
        self.reader.read()?;

        let user = self.parse_user()?;

        if user.is_some() {
            self.reader.read()?;
        }
        
        let host = self.parse_uri_host()?;
        let port = self.parse_port()?;

        Ok(RequestLine { method, uri: Uri::new(scheme, user, host, port)})

    }
}
