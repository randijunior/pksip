/*
Via               =  ( "Via" / "v" ) HCOLON via-parm *(COMMA via-parm)
via-parm          =  sent-protocol LWS sent-by *( SEMI via-params )
via-params        =  via-ttl / via-maddr
                     / via-received / via-branch
                     / via-extension
via-ttl           =  "ttl" EQUAL ttl
via-maddr         =  "maddr" EQUAL host
via-received      =  "received" EQUAL (IPv4address / IPv6address)
via-branch        =  "branch" EQUAL token
via-extension     =  generic-param
sent-protocol     =  protocol-name SLASH protocol-version
                     SLASH transport
protocol-name     =  "SIP" / token
protocol-version  =  token
transport         =  "UDP" / "TCP" / "TLS" / "SCTP"
                     / other-transport
sent-by           =  host [ COLON port ]
ttl               =  1*3DIGIT ; 0 to 255
*/

use super::SipHeaderParser;
use crate::{
    byte_reader::ByteReader,
    macros::{read_until_byte, sip_parse_error, space},
    msg::Transport,
    parser::{Result, SipParser},
    uri::{HostPort, Params},
};
use std::str;

#[derive(Debug, PartialEq, Eq, Default)]
pub struct ViaParams<'a> {
    ttl: Option<&'a str>,
    maddr: Option<&'a str>,
    received: Option<&'a str>,
    branch: Option<&'a str>,
    rport: Option<u16>,
}

impl<'a> ViaParams<'a> {
    pub fn set_branch(&mut self, branch: &'a str) {
        self.branch = Some(branch);
    }

    pub fn set_ttl(&mut self, ttl: &'a str) {
        self.ttl = Some(ttl);
    }
    pub fn set_maddr(&mut self, maddr: &'a str) {
        self.maddr = Some(maddr);
    }
    pub fn set_received(&mut self, received: &'a str) {
        self.received = Some(received);
    }
    pub fn set_rport(&mut self, rport: u16) {
        self.rport = Some(rport);
    }
}

// SIP Via Header
#[derive(Debug, PartialEq, Eq)]
pub struct Via<'a> {
    pub(crate) transport: Transport,
    pub(crate) sent_by: HostPort<'a>,
    pub(crate) params: Option<ViaParams<'a>>,
    pub(crate) comment: Option<&'a str>,
    pub(crate) others_params: Option<Params<'a>>,
}

impl<'a> SipHeaderParser<'a> for Via<'a> {
    const NAME: &'static [u8] = b"Via";
    const SHORT_NAME: Option<&'static [u8]> = Some(b"v");

    fn parse(reader: &mut ByteReader<'a>) -> Result<Self> {
        SipParser::parse_sip_version(reader)?;

        if reader.next() != Some(&b'/') {
            return sip_parse_error!("Invalid via Hdr!");
        }
        let bytes = read_until_byte!(reader, b' ');
        let transport = Transport::from(bytes);

        space!(reader);

        let sent_by = SipParser::parse_host(reader)?;
        let (params, others_params) = SipParser::parse_via_params(reader)?;

        let comment = if reader.peek() == Some(&b'(') {
            reader.next();
            let comment = read_until_byte!(reader, b')');
            reader.next();
            Some(str::from_utf8(comment)?)
        } else {
            None
        };

        Ok(Via {
            transport,
            sent_by,
            params,
            others_params,
            comment,
        })
    }
}
