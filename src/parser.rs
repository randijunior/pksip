use crate::{
    byte_reader::{ByteReader, ByteReaderError},
    macros::{
        alpha, b_map, digits, newline, next, peek, read_while, sip_parse_error,
        space, tag, until_byte, until_newline,
    },
    msg::{RequestLine, SipMethod, SipStatusCode, StatusLine},
    uri::{
        GenericParam, Host, Scheme, Uri, UriParam, UserInfo, LR_PARAM, MADDR_PARAM,
        METHOD_PARAM, TANSPORT_PARAM, TTL_PARAM, USER_PARAM,
    },
};

const SIPV2: &'static [u8] = "SIP/2.0".as_bytes();

type Result<T> = std::result::Result<T, SipParserError>;

use core::str;
use std::{
    collections::HashSet,
    net::IpAddr,
    str::{FromStr, Utf8Error},
};

// A-Z a-z 0-9 -_.!~*'() &=+$,;?/%
// For reading user part on sip uri.
const USER_SPEC_MAP: [bool; 256] = b_map![
// \0                            \n
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// \w  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _ 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  \x7f  
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,

// Extended ASCII (character code 128-255)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// A-Z a-z 0-9 -_.!~*'() &=+$,%
// For reading password part on sip uri.
const PASS_SPEC_MAP: [bool; 256] = b_map![
// \0                            \n
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// \w  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 0,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _ 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  \x7f  
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,

// Extended ASCII (character code 128-255)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

// A-Z a-z 0-9 -_.  tirar ~*'() &=+$,%
// For reading password part on sip uri.
const HOST_SPEC_MAP: [bool; 256] = b_map![
// \0                            \n
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// \w  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _ 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  \x7f  
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,

// Extended ASCII (character code 128-255)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];


// "[]/:&+$"  "-_.!~*'()" "%"
const PARAM_SPEC_MAP: [bool; 256] = b_map![
// \0                            \n
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
// \w  !  "  #  $  %  &  '  (  )  *  +  ,  -  .  /
    0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0,
//  0  1  2  3  4  5  6  7  8  9  :  ;  <  =  >  ?
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
//  @  A  B  C  D  E  F  G  H  I  J  K  L  M  N  O
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  P  Q  R  S  T  U  V  W  X  Y  Z  [  \  ]  ^  _ 
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
//  `  a  b  c  d  e  f  g  h  i  j  k  l  m  n  o  
    0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
//  p  q  r  s  t  u  v  w  x  y  z  {  |  }  ~  \x7f  
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,

// Extended ASCII (character code 128-255)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[derive(Debug, PartialEq)]
pub struct SipParserError {
    message: String,
}

impl From<Utf8Error> for SipParserError {
    fn from(value: Utf8Error) -> Self {
        SipParserError {
            message: format!("{:#?}", value),
        }
    }
}

impl<'a> From<ByteReaderError<'a>> for SipParserError {
    fn from(err: ByteReaderError) -> Self {
        SipParserError {
            message: format!(
                "Failed to parse at line:{},
                column:{},
                kind:{:?},
                input:'{}'",
                err.pos.line,
                err.pos.col,
                err.kind,
                String::from_utf8_lossy(err.input)
            ),
        }
    }
}

pub fn parse_status_line<'a>(reader: &mut ByteReader<'a>) -> Result<StatusLine<'a>> {
    let _ = tag!(reader, SIPV2);

    space!(reader);
    let digits = digits!(reader);
    space!(reader);

    let status_code = SipStatusCode::from(digits);
    let bytes = until_newline!(reader);

    let rp = str::from_utf8(bytes)?;

    newline!(reader);
    Ok(StatusLine::new(status_code, rp))
}

#[inline]
fn parse_scheme<'a>(reader: &mut ByteReader) -> Result<Scheme> {
    match until_byte!(reader, b':') {
        b"sip" => Ok(Scheme::Sip),
        b"sips" => Ok(Scheme::Sips),
        // Unsupported URI scheme
        _ => sip_parse_error!("Can't parse sip uri scheme"),
    }
}

#[inline]
fn is_user(b: u8) -> bool {
    USER_SPEC_MAP[b as usize]
}

#[inline]
fn is_pass(b: u8) -> bool {
    PASS_SPEC_MAP[b as usize]
}

#[inline(always)]
fn has_user(reader: &ByteReader) -> bool {
    let mut matched = None;
    for &byte in reader.as_ref().iter() {
        match byte {
            b'@' | b' ' | b'\n' | b'>' => {
                matched = Some(byte);
                break;
            }
            _ => continue,
        }
    }
    matched == Some(b'@')
}

fn parse_user<'a>(reader: &mut ByteReader<'a>) -> Result<Option<UserInfo<'a>>> {
    if has_user(reader) {
        let bytes = read_while!(reader, is_user);
        let name = str::from_utf8(bytes)?;

        if peek!(reader) == Some(b':') {
            next!(reader);
            let bytes = read_while!(reader, is_pass);
            next!(reader);

            Ok(Some(UserInfo {
                name,
                password: Some(str::from_utf8(bytes)?),
            }))
        } else {
            next!(reader);
            Ok(Some(UserInfo {
                name,
                password: None,
            }))
        }
    } else {
        Ok(None)
    }
}

fn parse_host<'a>(reader: &mut ByteReader<'a>) -> Result<Host<'a>> {
    if let Some(_) = reader.read_if(|b| b == b'[') {
        // the '[' and ']' characters are removed from the host
        next!(reader);
        let host = until_byte!(reader, b']');
        next!(reader);
        let host = str::from_utf8(host)?;
        if let Ok(host) = host.parse() {
            next!(reader);
            Ok(Host::IpAddr(IpAddr::V6(host)))
        } else {
            sip_parse_error!("Error parsing Ipv6 Host!")
        }
    } else {
        let host = read_while!(reader, |b| HOST_SPEC_MAP[b as usize]);
        let host = str::from_utf8(host)?;
        if let Ok(addr) = IpAddr::from_str(host) {
            Ok(Host::IpAddr(addr))
        } else {
            Ok(Host::DomainName(host))
        }
    }
}

fn parse_port<'a>(reader: &mut ByteReader) -> Result<Option<u16>> {
    if let Some(_) = reader.read_if(|b| b == b':') {
        let digits = digits!(reader);
        let digits = str::from_utf8(digits)?;

        match u16::from_str_radix(digits, 10) {
            Ok(port) => Ok(Some(port)),
            Err(_) => sip_parse_error!("Port is invalid integer!"),
        }
    } else {
        Ok(None)
    }
}

fn parse_uri_params<'a>(
    reader: &mut ByteReader<'a>,
    rfc_params: &mut HashSet<UriParam<'a>>,
    other_params: &mut Vec<GenericParam<'a>>
) -> Result<()> {
    while peek!(reader) == Some(b';') {
        next!(reader);
        let name = read_while!(reader, |b| PARAM_SPEC_MAP[b as usize]);
        let value = if peek!(reader) == Some(b'=') {
            next!(reader);
            let value = read_while!(reader, |b| PARAM_SPEC_MAP[b as usize]);
            str::from_utf8(value)?
        } else {
            ""
        };
        match name {
            USER_PARAM => {
                rfc_params.insert(UriParam::User(value));
            }
            METHOD_PARAM => {
                rfc_params.insert(UriParam::Method(value));
            }
            TANSPORT_PARAM => {
                rfc_params.insert(UriParam::Transport(value));
            }
            TTL_PARAM => {
                rfc_params.insert(UriParam::TTL(value));
            }
            LR_PARAM => {
                rfc_params.insert(UriParam::Lr(value));
            }
            MADDR_PARAM => {
                rfc_params.insert(UriParam::Maddr(value));
            }
            _ => {
                other_params.push(GenericParam {
                    name: str::from_utf8(name)?,
                    value,
                });
            }
        }
    }

    Ok(())
}


fn parse_sip_uri<'a>(reader: &mut ByteReader<'a>) -> Result<Uri<'a>> {
    let scheme = parse_scheme(reader)?;
    // take ':'
    next!(reader);

    let user = parse_user(reader)?;
    let host = parse_host(reader)?;
    let port = parse_port(reader)?;
    let mut rfc_params = HashSet::new();
    let mut other_params = vec![];
    parse_uri_params(reader, &mut rfc_params, &mut other_params)?;

    Ok(Uri {
        scheme,
        user,
        host,
        port,
        rfc_params,
        other_params,
    })

}


pub fn parse_request_line<'a>(
    reader: &mut ByteReader<'a>,
) -> Result<RequestLine<'a>> {
    let b_method = alpha!(reader);
    let method = SipMethod::from(b_method);

    space!(reader);
    let uri = parse_sip_uri(reader)?;

    space!(reader);

    let _ = tag!(reader, SIPV2);
    newline!(reader);

    Ok(RequestLine {
        method,
        uri,
    })
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_parse_status_line() {
        let sc_ok = SipStatusCode::Ok;
        let buf = "SIP/2.0 200 OK\r\n".as_bytes();
        let mut reader = ByteReader::new(buf);

        assert_eq!(
            parse_status_line(&mut reader),
            Ok(StatusLine {
                status_code: sc_ok,
                reason_phrase: sc_ok.reason_phrase()
            })
        );
        let sc_not_found = SipStatusCode::NotFound;
        let buf = "SIP/2.0 404 Not Found\r\n".as_bytes();
        let mut reader = ByteReader::new(buf);

        assert_eq!(
            parse_status_line(&mut reader),
            Ok(StatusLine {
                status_code: sc_not_found,
                reason_phrase: sc_not_found.reason_phrase()
            })
        );
    }

    #[test]
    fn test_req_status_line() {
        let msg = "REGISTER sip:1000b3@10.1.1.7:8089 SIP/2.0\r\n".as_bytes();
        let addr: IpAddr = "10.1.1.7".parse().unwrap();
        let mut reader = ByteReader::new(msg);
        assert_eq!(
            parse_request_line(&mut reader),
            Ok(RequestLine {
                method: SipMethod::Register,
                uri: Uri {
                    scheme: Scheme::Sip,
                    user: Some(UserInfo {
                        name: "1000b3",
                        password: None
                    }),
                    host: Host::IpAddr(addr),
                    port: Some(8089),
                    rfc_params: HashSet::new(),
                    other_params: vec![]
                }
            })
        );
    }
    #[test]
    fn status_line() {
        let msg =
            "REGISTER sip:alice@atlanta.com;maddr=239.255.255.1;ttl=15 SIP/2.0\r\n"
                .as_bytes();
        let mut reader = ByteReader::new(msg);
        println!("{:#?}", parse_request_line(&mut reader));
    }
}
