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
